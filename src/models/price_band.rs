/// Colour band for a unit price, shared by the chart, its legend and the headline price.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriceBand {
    Negative,
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
    Peak,
}

impl PriceBand {
    pub const ALL: [Self; 7] = [
        Self::Negative,
        Self::VeryLow,
        Self::Low,
        Self::Medium,
        Self::High,
        Self::VeryHigh,
        Self::Peak,
    ];

    /// The band a price (p/kWh, inc. VAT) falls into.
    pub fn for_price(price: f64) -> Self {
        if price < 0.0 {
            Self::Negative
        } else if price < 7.5 {
            Self::VeryLow
        } else if price < 11.25 {
            Self::Low
        } else if price < 15.0 {
            Self::Medium
        } else if price < 22.5 {
            Self::High
        } else if price < 30.0 {
            Self::VeryHigh
        } else {
            Self::Peak
        }
    }

    /// CSS class that sets `--band` to this band's colour.
    pub const fn css_class(self) -> &'static str {
        match self {
            Self::Negative => "band-negative",
            Self::VeryLow => "band-1",
            Self::Low => "band-2",
            Self::Medium => "band-3",
            Self::High => "band-4",
            Self::VeryHigh => "band-5",
            Self::Peak => "band-6",
        }
    }

    /// Legend label describing the band's price range.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Negative => "Below 0p",
            Self::VeryLow => "0–7.5p",
            Self::Low => "7.5–11.25p",
            Self::Medium => "11.25–15p",
            Self::High => "15–22.5p",
            Self::VeryHigh => "22.5–30p",
            Self::Peak => "30p+",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PriceBand;

    #[test]
    fn band_boundaries_are_inclusive_of_the_lower_bound() {
        assert_eq!(PriceBand::for_price(-0.01), PriceBand::Negative);
        assert_eq!(PriceBand::for_price(0.0), PriceBand::VeryLow);
        assert_eq!(PriceBand::for_price(7.5), PriceBand::Low);
        assert_eq!(PriceBand::for_price(11.25), PriceBand::Medium);
        assert_eq!(PriceBand::for_price(15.0), PriceBand::High);
        assert_eq!(PriceBand::for_price(22.5), PriceBand::VeryHigh);
        assert_eq!(PriceBand::for_price(30.0), PriceBand::Peak);
    }
}
