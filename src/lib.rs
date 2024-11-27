//! ta is a Rust library for technical analysis. It provides number of technical indicators
//! that can be used to build trading strategies for stock markets, futures, forex, cryptocurrencies, etc.
//!
//! Every indicator is implemented as a data structure with fields, that define parameters and
//! state.
//!
//! Every indicator implements [Next<T>](trait.Next.html) and [Reset](trait.Reset.html) traits,
//! which are the core concept of the library.
//!
//! Since `Next<T>` is a generic trait, most of the indicators can work with both input types: `f64` and more complex
//! structures like [`DataItem`](struct.DataItem.html).
//!
//! # List of indicators
//!
//! * Trend
//!   * [Exponential Moving Average (EMA)](crate::indicators::ExponentialMovingAverage)
//!   * [Simple Moving Average (SMA)](crate::indicators::SimpleMovingAverage)
//!   * [Weighted Moving Average (WMA)](crate::indicators::WeightedMovingAverage)
//! * Oscillators
//!   * [Relative Strength Index (RSI)](indicators/struct.RelativeStrengthIndex.html)
//!   * [Fast Stochastic](indicators/struct.FastStochastic.html)
//!   * [Slow Stochastic](indicators/struct.SlowStochastic.html)
//!   * [Moving Average Convergence Divergence (MACD)](indicators/struct.MovingAverageConvergenceDivergence.html)
//!   * [Percentage Price Oscillator (PPO)](indicators/struct.PercentagePriceOscillator.html)
//!   * [Commodity Channel Index (CCI)](indicators/struct.CommodityChannelIndex.html)
//!   * [Money Flow Index (MFI)](indicators/struct.MoneyFlowIndex.html)
//! * Other
//!   * [Standard Deviation (SD)](indicators/struct.StandardDeviation.html)
//!   * [Mean Absolute Deviation (MAD)](indicators/struct.MeanAbsoluteDeviation.html)
//!   * [Bollinger Bands (BB)](indicators/struct.BollingerBands.html)
//!   * [Chandelier Exit (CE)](indicators/struct.ChandelierExit.html)
//!   * [Keltner Channel (KC)](indicators/struct.KeltnerChannel.html)
//!   * [Maximum](indicators/struct.Maximum.html)
//!   * [Minimum](indicators/struct.Minimum.html)
//!   * [True Range](indicators/struct.TrueRange.html)
//!   * [Average True Range (ATR)](indicators/struct.AverageTrueRange.html)
//!   * [Efficiency Ratio (ER)](indicators/struct.EfficiencyRatio.html)
//!   * [Rate of Change (ROC)](indicators/struct.RateOfChange.html)
//!   * [On Balance Volume (OBV)](indicators/struct.OnBalanceVolume.html)
//!
#[macro_use]
mod helpers;

#[cfg(test)]
#[macro_use]
mod test_helper;

pub mod errors;
pub mod indicators;

mod traits;
pub use crate::traits::*;

mod data_item;
pub use crate::data_item::Candle;
pub use crate::data_item::CandleBuilder;