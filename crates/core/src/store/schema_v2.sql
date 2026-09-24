-- v2 : « Hospitalisation longue durée » fusionnée dans « Hospitalisation complète » (décision de Franck, 24/09/2026).
-- Le libellé source d'origine reste dans service_label_source et dans les révisions antérieures.
INSERT INTO audit_event(at, kind, entity, entity_id, field, old_value, new_value, reason, author)
SELECT strftime('%Y-%m-%dT%H:%M:%S', 'now', 'localtime'), 'migration', 'encounter', encounter_id, 'service_code', 'hospit_longue', 'hospit_complete',
       'Fusion des services décidée par le praticien (schéma v2)', 'migration'
FROM field_value WHERE field = 'service_code' AND value_text = 'hospit_longue';
UPDATE field_value SET value_text = 'hospit_complete' WHERE field = 'service_code' AND value_text = 'hospit_longue';
UPDATE encounter SET service_code = 'hospit_complete' WHERE service_code = 'hospit_longue';
