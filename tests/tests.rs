#[cfg(test)]
mod tests {
    use agile_dashboard::hooks::use_rates::DataState;
    use agile_dashboard::models::{
        error::AppError,
        rates::{Rate, Rates},
    };
    use chrono::{Duration, TimeZone, Utc};

    // Helper function to create test rates
    fn create_test_rates() -> Vec<Rate> {
        vec![
            Rate {
                value_inc_vat: 15.5,
                valid_from: Utc.with_ymd_and_hms(2025, 10, 4, 0, 0, 0).unwrap(),
                valid_to: Utc.with_ymd_and_hms(2025, 10, 4, 0, 30, 0).unwrap(),
            },
            Rate {
                value_inc_vat: 20.3,
                valid_from: Utc.with_ymd_and_hms(2025, 10, 4, 0, 30, 0).unwrap(),
                valid_to: Utc.with_ymd_and_hms(2025, 10, 4, 1, 0, 0).unwrap(),
            },
            Rate {
                value_inc_vat: 18.7,
                valid_from: Utc.with_ymd_and_hms(2025, 10, 4, 1, 0, 0).unwrap(),
                valid_to: Utc.with_ymd_and_hms(2025, 10, 4, 1, 30, 0).unwrap(),
            },
        ]
    }

    // Helper function to create rates with current timestamp
    fn create_current_time_rates() -> Vec<Rate> {
        let now = Utc::now();
        let past = now - Duration::minutes(15);
        let future = now + Duration::minutes(15);

        vec![
            Rate {
                value_inc_vat: 25.5,
                valid_from: past,
                valid_to: future,
            },
            Rate {
                value_inc_vat: 30.0,
                valid_from: future,
                valid_to: future + Duration::hours(1),
            },
        ]
    }

    // ===== Error Type Tests =====

    #[test]
    fn test_app_error_api_display() {
        let error = AppError::ApiError("Connection failed".to_string());
        assert_eq!(error.to_string(), "API Error: Connection failed");
    }

    #[test]
    fn test_app_error_data_display() {
        let error = AppError::DataError("Invalid data".to_string());
        assert_eq!(error.to_string(), "Data Error: Invalid data");
    }

    // ===== Rate Model Tests =====

    #[test]
    fn test_rate_deserialization() {
        let json = r#"{
            "value_inc_vat": 15.5,
            "valid_from": "2025-10-04T00:00:00Z",
            "valid_to": "2025-10-04T00:30:00Z"
        }"#;

        let rate: Result<Rate, _> = serde_json::from_str(json);
        assert!(rate.is_ok());

        let rate = rate.unwrap();
        assert_eq!(rate.value_inc_vat, 15.5);
    }

    #[test]
    fn test_rate_equality() {
        let rate1 = Rate {
            value_inc_vat: 15.5,
            valid_from: Utc.with_ymd_and_hms(2025, 10, 4, 0, 0, 0).unwrap(),
            valid_to: Utc.with_ymd_and_hms(2025, 10, 4, 0, 30, 0).unwrap(),
        };

        let rate2 = Rate {
            value_inc_vat: 15.5,
            valid_from: Utc.with_ymd_and_hms(2025, 10, 4, 0, 0, 0).unwrap(),
            valid_to: Utc.with_ymd_and_hms(2025, 10, 4, 0, 30, 0).unwrap(),
        };

        assert_eq!(rate1, rate2);
    }

    // ===== Rates Model Tests =====

    #[test]
    fn test_rates_new() {
        let rates_vec = create_test_rates();
        let rates = Rates::new(rates_vec.clone());

        // Verify construction works
        assert_eq!(rates.filter_for_today().len(), 0); // Test data is in the past
    }

    #[test]
    fn test_rates_current_price_found() {
        let rates_vec = create_current_time_rates();
        let rates = Rates::new(rates_vec);

        let result = rates.current_price();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 25.5);
    }

    #[test]
    fn test_rates_current_price_not_found() {
        let past_time = Utc::now() - Duration::hours(2);
        let older_time = past_time - Duration::hours(1);

        let rates_vec = vec![Rate {
            value_inc_vat: 15.5,
            valid_from: older_time,
            valid_to: past_time,
        }];

        let rates = Rates::new(rates_vec);
        let result = rates.current_price();

        assert!(result.is_err());
        match result {
            Err(AppError::DataError(msg)) => {
                assert!(msg.contains("No current rate found"));
            }
            _ => panic!("Expected DataError"),
        }
    }

    #[test]
    fn test_rates_stats_calculation() {
        let rates_vec = create_test_rates();
        let rates = Rates::new(rates_vec);

        let result = rates.stats();
        assert!(result.is_ok());

        let stats = result.unwrap();
        assert_eq!(stats.min, 15.5);
        assert_eq!(stats.max, 20.3);

        // Average: (15.5 + 20.3 + 18.7) / 3 = 18.166...
        assert!((stats.avg - 18.166666666666668).abs() < 0.0001);

        assert_eq!(stats.price_range, "15.50p - 20.30p");

        // Current price should be 0.0 since test data is in the past
        assert_eq!(stats.current, 0.0);
    }

    #[test]
    fn test_rates_stats_empty_data() {
        let rates = Rates::new(vec![]);
        let result = rates.stats();

        assert!(result.is_err());
        match result {
            Err(AppError::DataError(msg)) => {
                assert!(msg.contains("No data available"));
            }
            _ => panic!("Expected DataError"),
        }
    }

    #[test]
    fn test_rates_filter_for_today() {
        let now = Utc::now();
        let today_midnight = now.date_naive();

        let rates_vec = vec![
            Rate {
                value_inc_vat: 15.5,
                // Yesterday
                valid_from: Utc.from_utc_datetime(
                    &today_midnight
                        .checked_sub_days(chrono::Days::new(1))
                        .unwrap()
                        .and_hms_opt(12, 0, 0)
                        .unwrap(),
                ),
                valid_to: Utc.from_utc_datetime(
                    &today_midnight
                        .checked_sub_days(chrono::Days::new(1))
                        .unwrap()
                        .and_hms_opt(12, 30, 0)
                        .unwrap(),
                ),
            },
            Rate {
                value_inc_vat: 20.3,
                // Today
                valid_from: Utc.from_utc_datetime(&today_midnight.and_hms_opt(12, 0, 0).unwrap()),
                valid_to: Utc.from_utc_datetime(&today_midnight.and_hms_opt(12, 30, 0).unwrap()),
            },
        ];

        let rates = Rates::new(rates_vec);
        let today_rates = rates.filter_for_today();

        assert_eq!(today_rates.len(), 1);
        assert_eq!(today_rates[0].value_inc_vat, 20.3);
    }

    #[test]
    fn test_rates_series_data_format() {
        let now = Utc::now();
        let today_midnight = now.date_naive();

        let rates_vec = vec![
            Rate {
                value_inc_vat: 15.5,
                valid_from: Utc.from_utc_datetime(&today_midnight.and_hms_opt(0, 0, 0).unwrap()),
                valid_to: Utc.from_utc_datetime(&today_midnight.and_hms_opt(0, 30, 0).unwrap()),
            },
            Rate {
                value_inc_vat: 20.3,
                valid_from: Utc.from_utc_datetime(&today_midnight.and_hms_opt(0, 30, 0).unwrap()),
                valid_to: Utc.from_utc_datetime(&today_midnight.and_hms_opt(1, 0, 0).unwrap()),
            },
        ];

        let rates = Rates::new(rates_vec);
        let result = rates.series_data();

        assert!(result.is_ok());
        let (x_data, y_data) = result.unwrap();

        assert_eq!(x_data.len(), y_data.len());
        assert_eq!(x_data.len(), 2);
        assert_eq!(y_data[0], 15.5);
        assert_eq!(y_data[1], 20.3);

        // Check format includes date and time
        assert!(x_data[0].contains("00:00"));
    }

    #[test]
    fn test_rates_series_data_sorting() {
        let now = Utc::now();
        let today_midnight = now.date_naive();

        // Create rates in reverse chronological order
        let rates_vec = vec![
            Rate {
                value_inc_vat: 20.3,
                valid_from: Utc.from_utc_datetime(&today_midnight.and_hms_opt(1, 0, 0).unwrap()),
                valid_to: Utc.from_utc_datetime(&today_midnight.and_hms_opt(1, 30, 0).unwrap()),
            },
            Rate {
                value_inc_vat: 15.5,
                valid_from: Utc.from_utc_datetime(&today_midnight.and_hms_opt(0, 0, 0).unwrap()),
                valid_to: Utc.from_utc_datetime(&today_midnight.and_hms_opt(0, 30, 0).unwrap()),
            },
        ];

        let rates = Rates::new(rates_vec);
        let result = rates.series_data();

        assert!(result.is_ok());
        let (x_data, y_data) = result.unwrap();

        // Should be sorted by time, so 15.5 comes first
        assert_eq!(y_data[0], 15.5);
        assert_eq!(y_data[1], 20.3);
        assert!(x_data[0].contains("00:00"));
        assert!(x_data[1].contains("01:00"));
    }

    // ===== DataState Tests =====

    #[test]
    fn test_data_state_is_loading() {
        let loading = DataState::Loading;
        assert!(loading.is_loading());

        let loaded = DataState::Loaded(Rates::new(vec![]));
        assert!(!loaded.is_loading());

        let error = DataState::Error("Test error".to_string());
        assert!(!error.is_loading());
    }

    #[test]
    fn test_data_state_data_extraction() {
        let rates = Rates::new(create_test_rates());
        let loaded = DataState::Loaded(rates.clone());

        assert!(loaded.data().is_some());
        assert_eq!(loaded.data().unwrap(), &rates);

        let loading = DataState::Loading;
        assert!(loading.data().is_none());

        let error = DataState::Error("Test error".to_string());
        assert!(error.data().is_none());
    }

    #[test]
    fn test_data_state_equality() {
        let state1 = DataState::Loading;
        let state2 = DataState::Loading;
        assert_eq!(state1, state2);

        let state3 = DataState::Error("Test error".to_string());
        let state4 = DataState::Error("Test error".to_string());
        assert_eq!(state3, state4);

        let rates1 = Rates::new(create_test_rates());
        let rates2 = Rates::new(create_test_rates());
        let state5 = DataState::Loaded(rates1);
        let state6 = DataState::Loaded(rates2);
        assert_eq!(state5, state6);
    }
}
