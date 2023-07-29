use core::mem::{transmute, MaybeUninit};

pub use channel_config::ChannelConfig;
use stm32h7xx_hal::{
    adc,
    adc::Resolution,
    dma::{
        config::Priority,
        dma::{DmaConfig, StreamX},
        DBTransfer, Transfer,
    },
    stm32::{ADC1, DMA1},
};

const ADC_MAX_CHANNELS: usize = 16;
const ADC_MAX_RESOLUTION: f32 = 65536.0;

#[link_section = ".sram1_bss"]
#[no_mangle]
static mut ADC_DMA_BUFFER: MaybeUninit<[u16; ADC_MAX_CHANNELS]> = MaybeUninit::uninit();

type AdcDmaTransfer = stm32h7xx_hal::dma::Transfer<
    StreamX<DMA1, 2>,
    stm32h7xx_hal::adc::Adc<ADC1, adc::Enabled>,
    stm32h7xx_hal::dma::PeripheralToMemory,
    &'static mut [u16],
    DBTransfer,
>;

mod channel_config;

#[derive(Default)]
enum Oversampling {
    OvsNone,
    Ovs4,
    Ovs8,
    Ovs16,
    #[default]
    Ovs32,
    Ovs64,
    Ovs128,
    Ovs256,
    Ovs512,
    Ovs1024,
}

pub struct Adc {
    adc1: Option<adc::Adc<ADC1, adc::Enabled>>,
    channels: [u8; ADC_MAX_CHANNELS],
    channel_configs: [Option<ChannelConfig>; ADC_MAX_CHANNELS],
    dma_buffer: Option<&'static mut [u16; ADC_MAX_CHANNELS]>,
    dma_buffer_ptr: *mut u16,
    dma_stream: Option<StreamX<DMA1, 2>>,
    num_channels: usize,
    transfer: Option<AdcDmaTransfer>,
}

impl Adc {
    pub fn new(mut adc1: adc::Adc<ADC1, adc::Disabled>, dma1_stream2: StreamX<DMA1, 2>) -> Self {
        let dma_buffer: &'static mut [u16; ADC_MAX_CHANNELS] = {
            let buf: &mut [MaybeUninit<u16>; ADC_MAX_CHANNELS] =
                unsafe { transmute(&mut ADC_DMA_BUFFER) };
            for slot in buf.iter_mut() {
                unsafe {
                    slot.as_mut_ptr().write(0);
                }
            }
            unsafe { transmute(buf) }
        };

        let dma_buffer_ptr = &mut dma_buffer[0] as *mut u16;

        adc1.set_resolution(Resolution::SixteenBit);

        Self {
            adc1: Some(adc1.enable()),
            channels: [0; ADC_MAX_CHANNELS],
            channel_configs: [
                None, None, None, None, None, None, None, None, None, None, None, None, None, None,
                None, None,
            ],
            dma_buffer: Some(dma_buffer),
            dma_buffer_ptr,
            dma_stream: Some(dma1_stream2),
            num_channels: 0,
            transfer: None,
        }
    }

    pub fn init_single(&mut self, config: ChannelConfig) {
        self.channels[self.num_channels] = config.channel();
        self.channel_configs[self.num_channels] = Some(config);
        self.num_channels += 1;
    }

    pub fn start(&mut self) {
        let dma_buffer = self.dma_buffer.take().expect("TODO: type state");

        let mut transfer: AdcDmaTransfer = Transfer::init(
            self.dma_stream.take().unwrap(),
            self.adc1.take().unwrap(),
            &mut dma_buffer[..self.num_channels],
            None,
            self.config(),
        );

        transfer.start(|adc| {
            adc.start_conversion_dma_circ(&self.channels[..self.num_channels]);
        });

        self.transfer = Some(transfer);
    }

    fn config(&self) -> DmaConfig {
        DmaConfig::default()
            .priority(Priority::Low)
            .circular_buffer(true)
            .memory_increment(true)
    }

    pub fn stop() {
        // TODO
    }

    pub fn get_float(&mut self, channel: u8) -> f32 {
        self.get(channel) as f32 / ADC_MAX_RESOLUTION
    }

    pub fn get(&mut self, channel: u8) -> u16 {
        debug_assert!((channel as usize) < self.num_channels);

        unsafe {
            let location = self.dma_buffer_ptr.offset(channel as isize);
            core::ptr::read(location)
        }
    }
}
