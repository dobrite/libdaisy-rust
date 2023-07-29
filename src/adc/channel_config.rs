use embedded_hal::adc::Channel;
use stm32h7xx_hal::stm32::ADC1;

#[derive(Clone, Copy, Default)]
enum ConversionSpeed {
    Speed1Cycles5,
    Speed2Cycles5,
    #[default]
    Speed8Cycles5,
    Speed16Cycles5,
    Speed32Cycles5,
    Speed64Cycles5,
    Speed387Cycles5,
    Speed810Cycles5,
}

pub struct ChannelConfig {
    channel: u8,
    speed: ConversionSpeed,
}

impl ChannelConfig {
    pub fn new<Pin>(_pin: Pin) -> Self
    where
        Pin: Channel<ADC1, ID = u8>,
    {
        Self {
            channel: Pin::channel(),
            speed: ConversionSpeed::Speed8Cycles5,
        }
    }

    pub fn channel(&self) -> u8 {
        self.channel
    }
}
