-- Daily active installations (DAU), rolling weekly active (WAU), rolling monthly active (MAU),
-- and first-seen installations by UTC date. One installation is counted once per period.
WITH activity AS (
  SELECT installation_hash, activity_date
  FROM desktop_daily_active
  WHERE event_name = 'app_started'
),
activity_days AS (
  SELECT DISTINCT activity_date
  FROM activity
),
first_seen AS (
  SELECT installation_hash, MIN(activity_date) AS first_active_date
  FROM activity
  GROUP BY installation_hash
)
SELECT
  d.activity_date,
  COUNT(DISTINCT CASE
    WHEN a.activity_date = d.activity_date THEN a.installation_hash
  END) AS dau,
  COUNT(DISTINCT CASE
    WHEN a.activity_date >= date(d.activity_date, '-6 days')
      AND a.activity_date <= d.activity_date THEN a.installation_hash
  END) AS wau,
  COUNT(DISTINCT CASE
    WHEN a.activity_date >= date(d.activity_date, '-29 days')
      AND a.activity_date <= d.activity_date THEN a.installation_hash
  END) AS mau,
  COUNT(DISTINCT CASE
    WHEN f.first_active_date = d.activity_date THEN a.installation_hash
  END) AS first_seen_installations
FROM activity_days AS d
LEFT JOIN activity AS a
  ON a.activity_date BETWEEN date(d.activity_date, '-29 days') AND d.activity_date
LEFT JOIN first_seen AS f
  ON f.installation_hash = a.installation_hash
GROUP BY d.activity_date
ORDER BY d.activity_date DESC;

-- Strict D1/D7/D30 cohort retention. A cohort is the installations first observed
-- on a UTC date in the retained dataset. Immature cohorts return NULL, not 0.
WITH activity AS (
  SELECT installation_hash, activity_date
  FROM desktop_daily_active
  WHERE event_name = 'app_started'
),
cohorts AS (
  SELECT installation_hash, MIN(activity_date) AS cohort_date
  FROM activity
  GROUP BY installation_hash
),
retention AS (
  SELECT
    c.cohort_date,
    COUNT(*) AS cohort_size,
    SUM(EXISTS (
      SELECT 1 FROM activity AS a
      WHERE a.installation_hash = c.installation_hash
        AND a.activity_date = date(c.cohort_date, '+1 day')
    )) AS retained_d1,
    SUM(EXISTS (
      SELECT 1 FROM activity AS a
      WHERE a.installation_hash = c.installation_hash
        AND a.activity_date = date(c.cohort_date, '+7 days')
    )) AS retained_d7,
    SUM(EXISTS (
      SELECT 1 FROM activity AS a
      WHERE a.installation_hash = c.installation_hash
        AND a.activity_date = date(c.cohort_date, '+30 days')
    )) AS retained_d30
  FROM cohorts AS c
  GROUP BY c.cohort_date
)
SELECT
  cohort_date,
  cohort_size,
  CASE WHEN date(cohort_date, '+1 day') <= date('now')
    THEN ROUND(100.0 * retained_d1 / cohort_size, 1) END AS d1_retention_percent,
  CASE WHEN date(cohort_date, '+7 days') <= date('now')
    THEN ROUND(100.0 * retained_d7 / cohort_size, 1) END AS d7_retention_percent,
  CASE WHEN date(cohort_date, '+30 days') <= date('now')
    THEN ROUND(100.0 * retained_d30 / cohort_size, 1) END AS d30_retention_percent
FROM retention
ORDER BY cohort_date DESC;
