-- Seed data for 3D Quote Service
-- Initial service types and materials

-- Service Types
INSERT OR IGNORE INTO service_types (id, name, description, active)
VALUES
    ('3d_printing', 'Impression 3D', 'Service d''impression 3D FDM et résine', 1),
    ('laser_cutting', 'Découpe Laser', 'Service de découpe laser (futur)', 0),
    ('engraving', 'Gravure', 'Service de gravure (futur)', 0);

-- Materials for 3D Printing
INSERT OR IGNORE INTO materials (id, service_type_id, name, description, price_per_cm3, color, properties, active)
VALUES
    ('pla_white', '3d_printing', 'PLA Blanc', 'PLA standard blanc, biodégradable et facile à imprimer', 0.05, '#FFFFFF', '{"density": 1.24, "print_temperature": 200, "bed_temperature": 60, "strength": "medium", "flexibility": "low"}', 1),
    ('pla_black', '3d_printing', 'PLA Noir', 'PLA standard noir, biodégradable et facile à imprimer', 0.05, '#000000', '{"density": 1.24, "print_temperature": 200, "bed_temperature": 60, "strength": "medium", "flexibility": "low"}', 1),
    ('pla_red', '3d_printing', 'PLA Rouge', 'PLA standard rouge, biodégradable et facile à imprimer', 0.06, '#FF0000', '{"density": 1.24, "print_temperature": 200, "bed_temperature": 60, "strength": "medium", "flexibility": "low"}', 1),
    ('abs_white', '3d_printing', 'ABS Blanc', 'ABS résistant aux chocs et à la chaleur', 0.08, '#FFFFFF', '{"density": 1.04, "print_temperature": 240, "bed_temperature": 100, "strength": "high", "flexibility": "medium"}', 1),
    ('abs_black', '3d_printing', 'ABS Noir', 'ABS résistant aux chocs et à la chaleur', 0.08, '#000000', '{"density": 1.04, "print_temperature": 240, "bed_temperature": 100, "strength": "high", "flexibility": "medium"}', 1),
    ('petg_transparent', '3d_printing', 'PETG Transparent', 'PETG résistant et semi-flexible', 0.10, '#CCCCCC', '{"density": 1.27, "print_temperature": 230, "bed_temperature": 80, "strength": "high", "flexibility": "medium"}', 1),
    ('petg_blue', '3d_printing', 'PETG Bleu', 'PETG résistant et semi-flexible', 0.10, '#0000FF', '{"density": 1.27, "print_temperature": 230, "bed_temperature": 80, "strength": "high", "flexibility": "medium"}', 1),
    ('resin_standard', '3d_printing', 'Résine Standard', 'Résine photopolymère pour impressions détaillées', 0.15, '#FFCC00', '{"density": 1.10, "cure_time": 8, "strength": "medium", "flexibility": "low"}', 1),
    ('resin_tough', '3d_printing', 'Résine Tough', 'Résine renforcée pour pièces fonctionnelles', 0.20, '#808080', '{"density": 1.15, "cure_time": 10, "strength": "very_high", "flexibility": "low"}', 1),
    ('tpu_flexible', '3d_printing', 'TPU Flexible', 'Filament flexible pour pièces souples', 0.12, '#FF6600', '{"density": 1.21, "print_temperature": 220, "bed_temperature": 50, "strength": "medium", "flexibility": "very_high"}', 1);
