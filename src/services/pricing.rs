// Pricing service for quote calculation
use rust_decimal::Decimal;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};

/// Base fee for all quotes (handling, setup, etc.)
pub const BASE_FEE: f64 = 5.0;

/// Minimum order fee
pub const MINIMUM_ORDER_FEE: f64 = 10.0;

/// Quote item representing a single model's pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteItem {
    pub model_id: String,
    pub model_name: String,
    pub material_id: String,
    pub material_name: String,
    pub volume_cm3: f64,
    pub price_per_cm3: f64,
    pub material_cost: f64,
}

/// Complete quote breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteBreakdown {
    pub items: Vec<QuoteItem>,
    pub subtotal: f64,
    pub base_fee: f64,
    pub total: f64,
    pub minimum_applied: bool,
    pub calculated_total: f64,
}

/// Calculate price for a single model
pub fn calculate_model_price(volume_cm3: f64, price_per_cm3: f64) -> f64 {
    let price = Decimal::from_f64_retain(volume_cm3 * price_per_cm3)
        .unwrap_or_default()
        .round_dp(2);
    price.to_f64().unwrap_or(0.0)
}

/// Generate full quote breakdown from items
pub fn generate_quote_breakdown(items: Vec<QuoteItem>) -> QuoteBreakdown {
    let subtotal: f64 = items.iter().map(|item| item.material_cost).sum();

    let subtotal_rounded = Decimal::from_f64_retain(subtotal)
        .unwrap_or_default()
        .round_dp(2)
        .to_f64()
        .unwrap_or(0.0);

    let base_fee = BASE_FEE;

    let total = Decimal::from_f64_retain(subtotal_rounded + base_fee)
        .unwrap_or_default()
        .round_dp(2)
        .to_f64()
        .unwrap_or(0.0);

    // Apply minimum order fee if total is too low
    let minimum_applied = total < MINIMUM_ORDER_FEE;
    let final_total = if minimum_applied {
        MINIMUM_ORDER_FEE
    } else {
        total
    };

    QuoteBreakdown {
        items,
        subtotal: subtotal_rounded,
        base_fee,
        total: final_total,
        minimum_applied,
        calculated_total: total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_model_price_basic() {
        // 10 cm³ at 0.15 €/cm³ = 1.50 €
        let price = calculate_model_price(10.0, 0.15);
        assert_eq!(price, 1.5);
    }

    #[test]
    fn test_calculate_model_price_rounds_to_2_decimals() {
        // 3.333 cm³ at 0.123 €/cm³ = 0.409959 → 0.41 €
        let price = calculate_model_price(3.333, 0.123);
        assert_eq!(price, 0.41);
    }

    #[test]
    fn test_calculate_model_price_zero_volume() {
        let price = calculate_model_price(0.0, 0.15);
        assert_eq!(price, 0.0);
    }

    #[test]
    fn test_calculate_model_price_zero_price() {
        let price = calculate_model_price(10.0, 0.0);
        assert_eq!(price, 0.0);
    }

    #[test]
    fn test_calculate_model_price_large_volume() {
        // 1000 cm³ at 0.12 €/cm³ = 120 €
        let price = calculate_model_price(1000.0, 0.12);
        assert_eq!(price, 120.0);
    }

    #[test]
    fn test_calculate_model_price_small_values() {
        // Very small volume
        let price = calculate_model_price(0.001, 0.10);
        assert_eq!(price, 0.0); // Rounds to 0
    }

    #[test]
    fn test_generate_quote_breakdown_empty_items() {
        let breakdown = generate_quote_breakdown(vec![]);
        assert_eq!(breakdown.items.len(), 0);
        assert_eq!(breakdown.subtotal, 0.0);
        assert_eq!(breakdown.base_fee, BASE_FEE);
        assert_eq!(breakdown.calculated_total, BASE_FEE);
        assert!(breakdown.minimum_applied);
        assert_eq!(breakdown.total, MINIMUM_ORDER_FEE);
    }

    #[test]
    fn test_generate_quote_breakdown_single_item() {
        let items = vec![QuoteItem {
            model_id: "model1".to_string(),
            model_name: "Test Model".to_string(),
            material_id: "pla".to_string(),
            material_name: "PLA".to_string(),
            volume_cm3: 50.0,
            price_per_cm3: 0.12,
            material_cost: 6.0,
        }];

        let breakdown = generate_quote_breakdown(items);
        assert_eq!(breakdown.subtotal, 6.0);
        assert_eq!(breakdown.base_fee, 5.0);
        assert_eq!(breakdown.calculated_total, 11.0);
        assert!(!breakdown.minimum_applied);
        assert_eq!(breakdown.total, 11.0);
    }

    #[test]
    fn test_generate_quote_breakdown_multiple_items() {
        let items = vec![
            QuoteItem {
                model_id: "m1".to_string(),
                model_name: "Model 1".to_string(),
                material_id: "pla".to_string(),
                material_name: "PLA".to_string(),
                volume_cm3: 10.0,
                price_per_cm3: 0.12,
                material_cost: 1.2,
            },
            QuoteItem {
                model_id: "m2".to_string(),
                model_name: "Model 2".to_string(),
                material_id: "abs".to_string(),
                material_name: "ABS".to_string(),
                volume_cm3: 20.0,
                price_per_cm3: 0.15,
                material_cost: 3.0,
            },
            QuoteItem {
                model_id: "m3".to_string(),
                model_name: "Model 3".to_string(),
                material_id: "petg".to_string(),
                material_name: "PETG".to_string(),
                volume_cm3: 15.0,
                price_per_cm3: 0.14,
                material_cost: 2.1,
            },
        ];

        let breakdown = generate_quote_breakdown(items);
        // 1.2 + 3.0 + 2.1 = 6.3
        assert_eq!(breakdown.subtotal, 6.3);
        assert_eq!(breakdown.base_fee, 5.0);
        // 6.3 + 5.0 = 11.3
        assert_eq!(breakdown.calculated_total, 11.3);
        assert!(!breakdown.minimum_applied);
        assert_eq!(breakdown.total, 11.3);
        assert_eq!(breakdown.items.len(), 3);
    }

    #[test]
    fn test_generate_quote_breakdown_applies_minimum_order() {
        let items = vec![QuoteItem {
            model_id: "small".to_string(),
            model_name: "Tiny Model".to_string(),
            material_id: "pla".to_string(),
            material_name: "PLA".to_string(),
            volume_cm3: 1.0,
            price_per_cm3: 0.12,
            material_cost: 0.12,
        }];

        let breakdown = generate_quote_breakdown(items);
        assert_eq!(breakdown.subtotal, 0.12);
        assert_eq!(breakdown.base_fee, 5.0);
        // 0.12 + 5.0 = 5.12 < 10.0 minimum
        assert_eq!(breakdown.calculated_total, 5.12);
        assert!(breakdown.minimum_applied);
        assert_eq!(breakdown.total, MINIMUM_ORDER_FEE);
    }

    #[test]
    fn test_generate_quote_breakdown_exact_minimum() {
        // Create an item that when added to base fee equals exactly the minimum
        // Subtotal + BASE_FEE = MINIMUM_ORDER_FEE
        // Subtotal = 10.0 - 5.0 = 5.0
        let items = vec![QuoteItem {
            model_id: "exact".to_string(),
            model_name: "Exact Model".to_string(),
            material_id: "pla".to_string(),
            material_name: "PLA".to_string(),
            volume_cm3: 50.0,
            price_per_cm3: 0.10,
            material_cost: 5.0,
        }];

        let breakdown = generate_quote_breakdown(items);
        assert_eq!(breakdown.subtotal, 5.0);
        assert_eq!(breakdown.calculated_total, 10.0);
        // Equal to minimum, so minimum not applied
        assert!(!breakdown.minimum_applied);
        assert_eq!(breakdown.total, 10.0);
    }

    #[test]
    fn test_generate_quote_breakdown_just_below_minimum() {
        let items = vec![QuoteItem {
            model_id: "below".to_string(),
            model_name: "Below Minimum".to_string(),
            material_id: "pla".to_string(),
            material_name: "PLA".to_string(),
            volume_cm3: 49.9,
            price_per_cm3: 0.10,
            material_cost: 4.99,
        }];

        let breakdown = generate_quote_breakdown(items);
        // 4.99 + 5.0 = 9.99 < 10.0
        assert!(breakdown.calculated_total < MINIMUM_ORDER_FEE);
        assert!(breakdown.minimum_applied);
        assert_eq!(breakdown.total, MINIMUM_ORDER_FEE);
    }

    #[test]
    fn test_generate_quote_breakdown_preserves_precision() {
        // Test that decimal precision is properly maintained
        let items = vec![QuoteItem {
            model_id: "precision".to_string(),
            model_name: "Precision Test".to_string(),
            material_id: "resin".to_string(),
            material_name: "Resin".to_string(),
            volume_cm3: 33.33,
            price_per_cm3: 0.333,
            material_cost: 11.1,
        }];

        let breakdown = generate_quote_breakdown(items);
        assert_eq!(breakdown.subtotal, 11.1);
        assert_eq!(breakdown.calculated_total, 16.1);
        assert_eq!(breakdown.total, 16.1);
    }

    #[test]
    fn test_base_fee_constant() {
        assert_eq!(BASE_FEE, 5.0);
    }

    #[test]
    fn test_minimum_order_fee_constant() {
        assert_eq!(MINIMUM_ORDER_FEE, 10.0);
    }
}
