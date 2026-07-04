BEGIN;

INSERT INTO "user" (name, login) VALUES
  ('Alice Martin', 'alice'),
  ('Bob', 'bob1'),
  ('Bob', 'bob2'),
  ('Bob', 'bob3'),
  ('Caroline Petit', 'caroline');

INSERT INTO project (author_id, name, description) VALUES
  ((SELECT id FROM "user" WHERE login = 'alice' LIMIT 1), 'Website Redesign', 'Refonte complète du site vitrine'),
  ((SELECT id FROM "user" WHERE login = 'bob2' LIMIT 1), 'Mobile App', 'Développement de l application mobile iOS/Android'),
  ((SELECT id FROM "user" WHERE login = 'caroline' LIMIT 1), 'Data Pipeline', 'Pipeline ETL pour rapports mensuels');

INSERT INTO task (author_id, project_id, name, deadline) VALUES
  ((SELECT id FROM "user" WHERE login = 'alice' LIMIT 1),
   (SELECT id FROM project WHERE name = 'Website Redesign' LIMIT 1),
   'Design homepage', '2026-05-15T12:00:00Z'),
  ((SELECT id FROM "user" WHERE login = 'alice' LIMIT 1),
   (SELECT id FROM project WHERE name = 'Website Redesign' LIMIT 1),
   'Implement responsive layout', '2026-06-01T12:00:00Z'),
  ((SELECT id FROM "user" WHERE login = 'bob1' LIMIT 1),
   (SELECT id FROM project WHERE name = 'Mobile App' LIMIT 1),
   'Setup CI/CD', '2026-05-20T12:00:00Z'),
  ((SELECT id FROM "user" WHERE login = 'bob3' LIMIT 1),
   (SELECT id FROM project WHERE name = 'Mobile App' LIMIT 1),
   'Integrate auth', NULL),
  ((SELECT id FROM "user" WHERE login = 'caroline' LIMIT 1),
   (SELECT id FROM project WHERE name = 'Data Pipeline' LIMIT 1),
   'Schema design', '2026-04-30T12:00:00Z'),
  ((SELECT id FROM "user" WHERE login = 'caroline' LIMIT 1),
   NULL,
   'Prepare project proposal', '2026-04-25T12:00:00Z');

COMMIT;
