-- Seed data for development and testing
-- Inserts sample law_entries and jobs to verify the pipeline and admin UI

-- First insert jobs (law_entries reference them via foreign keys)

-- Completed harvest jobs
INSERT INTO jobs (id, job_type, law_id, status, priority, attempts, started_at, completed_at, result)
VALUES
    ('a0000001-0000-0000-0000-000000000001', 'harvest', 'zorgtoeslagwet', 'completed', 80, 1, now() - interval '2 hours', now() - interval '1 hour', '{"articles": 12}'),
    ('a0000001-0000-0000-0000-000000000002', 'harvest', 'zorgverzekeringswet', 'completed', 80, 1, now() - interval '3 hours', now() - interval '2 hours', '{"articles": 45}'),
    ('a0000001-0000-0000-0000-000000000003', 'harvest', 'awir', 'completed', 70, 1, now() - interval '4 hours', now() - interval '3 hours', '{"articles": 38}'),
    ('a0000001-0000-0000-0000-000000000004', 'harvest', 'wet_ib_2001', 'completed', 60, 2, now() - interval '5 hours', now() - interval '4 hours', '{"articles": 156}'),
    ('a0000001-0000-0000-0000-000000000005', 'harvest', 'participatiewet', 'completed', 70, 1, now() - interval '6 hours', now() - interval '5 hours', '{"articles": 78}');

-- Completed enrich jobs
INSERT INTO jobs (id, job_type, law_id, status, priority, attempts, started_at, completed_at, result)
VALUES
    ('b0000001-0000-0000-0000-000000000001', 'enrich', 'zorgtoeslagwet', 'completed', 80, 1, now() - interval '50 minutes', now() - interval '20 minutes', '{"machine_readable_articles": 8}'),
    ('b0000001-0000-0000-0000-000000000002', 'enrich', 'zorgverzekeringswet', 'completed', 80, 1, now() - interval '1 hour', now() - interval '40 minutes', '{"machine_readable_articles": 12}');

-- In-progress jobs
INSERT INTO jobs (id, job_type, law_id, status, priority, attempts, started_at)
VALUES
    ('c0000001-0000-0000-0000-000000000001', 'enrich', 'awir', 'processing', 70, 1, now() - interval '10 minutes'),
    ('c0000001-0000-0000-0000-000000000002', 'harvest', 'wmo_2015', 'processing', 60, 1, now() - interval '5 minutes');

-- Failed jobs
INSERT INTO jobs (id, job_type, law_id, status, priority, attempts, started_at, completed_at, result)
VALUES
    ('d0000001-0000-0000-0000-000000000001', 'harvest', 'jeugdwet', 'failed', 50, 3, now() - interval '1 hour', now() - interval '30 minutes', '{"error": "Connection timeout to wetten.overheid.nl"}'),
    ('d0000001-0000-0000-0000-000000000002', 'enrich', 'wet_ib_2001', 'failed', 60, 2, now() - interval '2 hours', now() - interval '1 hour', '{"error": "Article structure too complex for automated enrichment"}');

-- Pending jobs
INSERT INTO jobs (id, job_type, law_id, status, priority)
VALUES
    ('e0000001-0000-0000-0000-000000000001', 'harvest', 'huisvestingswet_2014', 'pending', 40),
    ('e0000001-0000-0000-0000-000000000002', 'harvest', 'woningwet', 'pending', 40),
    ('e0000001-0000-0000-0000-000000000003', 'enrich', 'participatiewet', 'pending', 70),
    ('e0000001-0000-0000-0000-000000000004', 'harvest', 'wet_langdurige_zorg', 'pending', 50),
    ('e0000001-0000-0000-0000-000000000005', 'enrich', 'wet_ib_2001', 'pending', 60);

-- Now insert law_entries referencing the jobs

-- Fully enriched laws
INSERT INTO law_entries (law_id, law_name, status, harvest_job_id, enrich_job_id, quality_score)
VALUES
    ('zorgtoeslagwet', 'Wet op de zorgtoeslag', 'enriched',
     'a0000001-0000-0000-0000-000000000001', 'b0000001-0000-0000-0000-000000000001', 0.92),
    ('zorgverzekeringswet', 'Zorgverzekeringswet', 'enriched',
     'a0000001-0000-0000-0000-000000000002', 'b0000001-0000-0000-0000-000000000002', 0.85);

-- Currently being enriched
INSERT INTO law_entries (law_id, law_name, status, harvest_job_id, enrich_job_id, quality_score)
VALUES
    ('awir', 'Algemene wet inkomensafhankelijke regelingen', 'enriching',
     'a0000001-0000-0000-0000-000000000003', 'c0000001-0000-0000-0000-000000000001', NULL);

-- Harvested, awaiting enrichment
INSERT INTO law_entries (law_id, law_name, status, harvest_job_id, quality_score)
VALUES
    ('wet_ib_2001', 'Wet inkomstenbelasting 2001', 'harvested',
     'a0000001-0000-0000-0000-000000000004', NULL),
    ('participatiewet', 'Participatiewet', 'harvested',
     'a0000001-0000-0000-0000-000000000005', NULL);

-- Currently being harvested
INSERT INTO law_entries (law_id, law_name, status)
VALUES
    ('wmo_2015', 'Wet maatschappelijke ondersteuning 2015', 'harvesting');

-- Harvest failed
INSERT INTO law_entries (law_id, law_name, status)
VALUES
    ('jeugdwet', 'Jeugdwet', 'harvest_failed');

-- Enrich failed
INSERT INTO law_entries (law_id, law_name, status, harvest_job_id, quality_score)
VALUES
    ('wet_ib_2001_v2', 'Wet inkomstenbelasting 2001 (herpoging)', 'enrich_failed',
     'a0000001-0000-0000-0000-000000000004', 0.45);

-- Queued for processing
INSERT INTO law_entries (law_id, law_name, status)
VALUES
    ('huisvestingswet_2014', 'Huisvestingswet 2014', 'queued'),
    ('woningwet', 'Woningwet', 'queued'),
    ('wet_langdurige_zorg', 'Wet langdurige zorg', 'queued');

-- Unknown/new entries
INSERT INTO law_entries (law_id, law_name, status)
VALUES
    ('wet_kinderopvang', 'Wet kinderopvang', 'unknown'),
    ('aow', 'Algemene Ouderdomswet', 'unknown'),
    ('ww', 'Werkloosheidswet', 'unknown');
