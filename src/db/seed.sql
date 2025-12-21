-- Seed data for 3D Quote Service
-- Initial service types and materials

-- Service Types
INSERT INTO service_types (id, name, description, active, created_at)
VALUES
    ('3d_printing', 'Impression 3D', 'Service d''impression 3D FDM et résine', true, '2025-01-01T00:00:00Z'),
    ('laser_cutting', 'Découpe Laser', 'Service de découpe laser (futur)', false, '2025-01-01T00:00:00Z'),
    ('engraving', 'Gravure', 'Service de gravure (futur)', false, '2025-01-01T00:00:00Z')
ON CONFLICT (id) DO NOTHING;

-- Materials for 3D Printing
INSERT INTO materials (
    id,
    service_type_id,
    name,
    description,
    price_per_cm3,
    color,
    properties,
    active,
    created_at,
    updated_at
)
VALUES
    ('pla_white', '3d_printing', 'PLA Blanc',
     'PLA standard blanc, biodégradable et facile à imprimer',
     0.050, '#FFFFFF',
     '{"density": 1.24, "print_temperature": 200, "bed_temperature": 60, "strength": "medium", "flexibility": "low"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('pla_black', '3d_printing', 'PLA Noir',
     'PLA standard noir, biodégradable et facile à imprimer',
     0.050, '#000000',
     '{"density": 1.24, "print_temperature": 200, "bed_temperature": 60, "strength": "medium", "flexibility": "low"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('abs_white', '3d_printing', 'ABS Blanc',
     'ABS résistant aux chocs et à la chaleur',
     0.080, '#F5F5F5',
     '{"density": 1.04, "print_temperature": 240, "bed_temperature": 100, "strength": "high", "flexibility": "medium"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('abs_black', '3d_printing', 'ABS Noir',
     'ABS résistant aux chocs et à la chaleur',
     0.080, '#1A1A1A',
     '{"density": 1.04, "print_temperature": 240, "bed_temperature": 100, "strength": "high", "flexibility": "medium"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('petg_transparent', '3d_printing', 'PETG Transparent',
     'PETG résistant et semi-flexible',
     0.090, '#CCCCCC',
     '{"density": 1.27, "print_temperature": 230, "bed_temperature": 80, "strength": "high", "flexibility": "medium"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('petg_blue', '3d_printing', 'PETG Bleu',
     'PETG résistant et semi-flexible',
     0.090, '#0066CC',
     '{"density": 1.27, "print_temperature": 230, "bed_temperature": 80, "strength": "high", "flexibility": "medium"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('asa_grey', '3d_printing', 'ASA Gris',
     'ASA résistant aux UV et aux intempéries',
     0.090, '#808080',
     '{"density": 1.07, "print_temperature": 245, "bed_temperature": 100, "strength": "high", "flexibility": "medium", "weather_resistant": true}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('tpu_flexible', '3d_printing', 'TPU Flexible',
     'Filament flexible pour pièces souples',
     0.120, '#FF6600',
     '{"density": 1.21, "print_temperature": 220, "bed_temperature": 50, "strength": "medium", "flexibility": "very_high"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('resin_standard', '3d_printing', 'Résine Standard',
     'Résine photopolymère pour impressions détaillées',
     0.150, '#F0E68C',
     '{"density": 1.10, "cure_time": 8, "strength": "medium", "flexibility": "low"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z'),

    ('resin_tough', '3d_printing', 'Résine Tough',
     'Résine renforcée pour pièces fonctionnelles',
     0.200, '#778899',
     '{"density": 1.15, "cure_time": 10, "strength": "very_high", "flexibility": "low"}',
     true, '2025-01-01T00:00:00Z', '2025-01-01T00:00:00Z')

ON CONFLICT (id) DO NOTHING;
