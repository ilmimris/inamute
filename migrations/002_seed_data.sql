-- Seed data for LRT Jabodebek
-- Based on official schedule images from @lrt_jabodebek

-- Stations
INSERT INTO stations (station_code, name, line, latitude, longitude) VALUES
('DA', 'Dukuh Atas', 'LRT_Jabodebek', -6.1936, 106.8226),
('SU', 'Setiabudi', 'LRT_Jabodebek', -6.1986, 106.8305),
('RS', 'Rasuna Said', 'LRT_Jabodebek', -6.2062, 106.8419),
('KU', 'Kuningan', 'LRT_Jabodebek', -6.2117, 106.8429),
('PC', 'Pancoran', 'LRT_Jabodebek', -6.2263, 106.8363),
('CK', 'Cikoko', 'LRT_Jabodebek', -6.2312, 106.8404),
('CI', 'Ciling', 'LRT_Jabodebek', -6.2385, 106.8543),
('CW', 'Cawang', 'LRT_Jabodebek', -6.2564, 106.8731),
('KR', 'Kampung Rambutan', 'LRT_Jabodebek', -6.2796, 106.8954),
('CP', 'Cipayung', 'LRT_Jabodebek', -6.2938, 106.9123),
('JM', 'Jati Mulya', 'LRT_Jabodebek', -6.3045, 106.9245),
('HT', 'Harjamukti', 'LRT_Jabodebek', -6.3123, 106.9356),
('HB', 'Halim', 'LRT_Jabodebek', -6.2703, 106.8967),
('JB', 'Jatibening', 'LRT_Jabodebek', -6.2834, 106.9078),
('BB', 'Jatibening Baru', 'LRT_Jabodebek', -6.2912, 106.9189);

-- Routes
INSERT INTO routes (route_code, name, description) VALUES
('LRT_JM_DA', 'Jati Mulya - Dukuh Atas', 'LRT Jabodebek corridor Jati Mulya to Dukuh Atas'),
('LRT_DA_JM', 'Dukuh Atas - Jati Mulya', 'LRT Jabodebek corridor Dukuh Atas to Jati Mulya (reverse)');

-- Route stops: Jati Mulya → Dukuh Atas
INSERT INTO route_stops (route_id, station_id, stop_order) VALUES
(1, 11, 1),  -- Jati Mulya
(1, 10, 2),  -- Cipayung
(1, 9, 3),   -- Kampung Rambutan
(1, 8, 4),   -- Cawang
(1, 6, 5),   -- Cikoko
(1, 5, 6),   -- Pancoran
(1, 4, 7),   -- Kuningan
(1, 3, 8),   -- Rasuna Said
(1, 2, 9),   -- Setiabudi
(1, 1, 10);  -- Dukuh Atas

-- Route stops: Dukuh Atas → Jati Mulya (reverse)
INSERT INTO route_stops (route_id, station_id, stop_order) VALUES
(2, 1, 1),   -- Dukuh Atas
(2, 2, 2),   -- Setiabudi
(2, 3, 3),   -- Rasuna Said
(2, 4, 4),   -- Kuningan
(2, 5, 5),   -- Pancoran
(2, 6, 6),   -- Cikoko
(2, 8, 7),   -- Cawang
(2, 9, 8),   -- Kampung Rambutan
(2, 10, 9),  -- Cipayung
(2, 11, 10); -- Jati Mulya

-- Weekend schedules: Jati Mulya → Dukuh Atas
-- Departures from Jati Mulya every ~20-30 min from 07:28 to 20:30
INSERT INTO schedules (route_id, station_id, departure_time, schedule_type, direction) VALUES
-- Jati Mulya departures (weekend)
(1, 11, '07:28', 'weekend', 'Dukuh Atas'),
(1, 11, '07:58', 'weekend', 'Dukuh Atas'),
(1, 11, '08:28', 'weekend', 'Dukuh Atas'),
(1, 11, '08:58', 'weekend', 'Dukuh Atas'),
(1, 11, '09:28', 'weekend', 'Dukuh Atas'),
(1, 11, '09:58', 'weekend', 'Dukuh Atas'),
(1, 11, '10:28', 'weekend', 'Dukuh Atas'),
(1, 11, '10:58', 'weekend', 'Dukuh Atas'),
(1, 11, '11:28', 'weekend', 'Dukuh Atas'),
(1, 11, '11:58', 'weekend', 'Dukuh Atas'),
(1, 11, '12:28', 'weekend', 'Dukuh Atas'),
(1, 11, '12:58', 'weekend', 'Dukuh Atas'),
(1, 11, '13:28', 'weekend', 'Dukuh Atas'),
(1, 11, '13:58', 'weekend', 'Dukuh Atas'),
(1, 11, '14:28', 'weekend', 'Dukuh Atas'),
(1, 11, '14:58', 'weekend', 'Dukuh Atas'),
(1, 11, '15:28', 'weekend', 'Dukuh Atas'),
(1, 11, '15:58', 'weekend', 'Dukuh Atas'),
(1, 11, '16:28', 'weekend', 'Dukuh Atas'),
(1, 11, '16:58', 'weekend', 'Dukuh Atas'),
(1, 11, '17:28', 'weekend', 'Dukuh Atas'),
(1, 11, '17:58', 'weekend', 'Dukuh Atas'),
(1, 11, '18:28', 'weekend', 'Dukuh Atas'),
(1, 11, '18:58', 'weekend', 'Dukuh Atas'),
(1, 11, '19:28', 'weekend', 'Dukuh Atas'),
(1, 11, '19:58', 'weekend', 'Dukuh Atas'),
(1, 11, '20:28', 'weekend', 'Dukuh Atas');

-- Weekday schedules: Jati Mulya → Dukuh Atas
-- More frequent during peak hours (every 10-15 min), every 20 min off-peak
INSERT INTO schedules (route_id, station_id, departure_time, schedule_type, direction) VALUES
-- Morning peak (05:30 - 09:00)
(1, 11, '05:30', 'weekday', 'Dukuh Atas'),
(1, 11, '05:45', 'weekday', 'Dukuh Atas'),
(1, 11, '06:00', 'weekday', 'Dukuh Atas'),
(1, 11, '06:08', 'weekday', 'Dukuh Atas'),
(1, 11, '06:15', 'weekday', 'Dukuh Atas'),
(1, 11, '06:22', 'weekday', 'Dukuh Atas'),
(1, 11, '06:30', 'weekday', 'Dukuh Atas'),
(1, 11, '06:38', 'weekday', 'Dukuh Atas'),
(1, 11, '06:45', 'weekday', 'Dukuh Atas'),
(1, 11, '06:52', 'weekday', 'Dukuh Atas'),
(1, 11, '07:00', 'weekday', 'Dukuh Atas'),
(1, 11, '07:08', 'weekday', 'Dukuh Atas'),
(1, 11, '07:15', 'weekday', 'Dukuh Atas'),
(1, 11, '07:22', 'weekday', 'Dukuh Atas'),
(1, 11, '07:30', 'weekday', 'Dukuh Atas'),
(1, 11, '07:38', 'weekday', 'Dukuh Atas'),
(1, 11, '07:45', 'weekday', 'Dukuh Atas'),
(1, 11, '07:52', 'weekday', 'Dukuh Atas'),
(1, 11, '08:00', 'weekday', 'Dukuh Atas'),
(1, 11, '08:08', 'weekday', 'Dukuh Atas'),
(1, 11, '08:15', 'weekday', 'Dukuh Atas'),
(1, 11, '08:22', 'weekday', 'Dukuh Atas'),
(1, 11, '08:30', 'weekday', 'Dukuh Atas'),
-- Midday (09:00 - 16:00) - every 20 min
(1, 11, '09:00', 'weekday', 'Dukuh Atas'),
(1, 11, '09:20', 'weekday', 'Dukuh Atas'),
(1, 11, '09:40', 'weekday', 'Dukuh Atas'),
(1, 11, '10:00', 'weekday', 'Dukuh Atas'),
(1, 11, '10:20', 'weekday', 'Dukuh Atas'),
(1, 11, '10:40', 'weekday', 'Dukuh Atas'),
(1, 11, '11:00', 'weekday', 'Dukuh Atas'),
(1, 11, '11:20', 'weekday', 'Dukuh Atas'),
(1, 11, '11:40', 'weekday', 'Dukuh Atas'),
(1, 11, '12:00', 'weekday', 'Dukuh Atas'),
(1, 11, '12:20', 'weekday', 'Dukuh Atas'),
(1, 11, '12:40', 'weekday', 'Dukuh Atas'),
(1, 11, '13:00', 'weekday', 'Dukuh Atas'),
(1, 11, '13:20', 'weekday', 'Dukuh Atas'),
(1, 11, '13:40', 'weekday', 'Dukuh Atas'),
(1, 11, '14:00', 'weekday', 'Dukuh Atas'),
(1, 11, '14:20', 'weekday', 'Dukuh Atas'),
(1, 11, '14:40', 'weekday', 'Dukuh Atas'),
(1, 11, '15:00', 'weekday', 'Dukuh Atas'),
(1, 11, '15:20', 'weekday', 'Dukuh Atas'),
(1, 11, '15:40', 'weekday', 'Dukuh Atas'),
-- Evening peak (16:00 - 19:00)
(1, 11, '16:00', 'weekday', 'Dukuh Atas'),
(1, 11, '16:08', 'weekday', 'Dukuh Atas'),
(1, 11, '16:15', 'weekday', 'Dukuh Atas'),
(1, 11, '16:22', 'weekday', 'Dukuh Atas'),
(1, 11, '16:30', 'weekday', 'Dukuh Atas'),
(1, 11, '16:38', 'weekday', 'Dukuh Atas'),
(1, 11, '16:45', 'weekday', 'Dukuh Atas'),
(1, 11, '16:52', 'weekday', 'Dukuh Atas'),
(1, 11, '17:00', 'weekday', 'Dukuh Atas'),
(1, 11, '17:08', 'weekday', 'Dukuh Atas'),
(1, 11, '17:15', 'weekday', 'Dukuh Atas'),
(1, 11, '17:22', 'weekday', 'Dukuh Atas'),
(1, 11, '17:30', 'weekday', 'Dukuh Atas'),
(1, 11, '17:38', 'weekday', 'Dukuh Atas'),
(1, 11, '17:45', 'weekday', 'Dukuh Atas'),
(1, 11, '17:52', 'weekday', 'Dukuh Atas'),
(1, 11, '18:00', 'weekday', 'Dukuh Atas'),
(1, 11, '18:08', 'weekday', 'Dukuh Atas'),
(1, 11, '18:15', 'weekday', 'Dukuh Atas'),
(1, 11, '18:22', 'weekday', 'Dukuh Atas'),
(1, 11, '18:30', 'weekday', 'Dukuh Atas'),
-- Night (19:00 - 22:00) - every 30 min
(1, 11, '19:00', 'weekday', 'Dukuh Atas'),
(1, 11, '19:30', 'weekday', 'Dukuh Atas'),
(1, 11, '20:00', 'weekday', 'Dukuh Atas'),
(1, 11, '20:30', 'weekday', 'Dukuh Atas'),
(1, 11, '21:00', 'weekday', 'Dukuh Atas'),
(1, 11, '21:30', 'weekday', 'Dukuh Atas'),
(1, 11, '22:00', 'weekday', 'Dukuh Atas');

-- Weekend schedules: Dukuh Atas → Jati Mulya (reverse direction)
INSERT INTO schedules (route_id, station_id, departure_time, schedule_type, direction) VALUES
(2, 1, '06:04', 'weekend', 'Jati Mulya'),
(2, 1, '06:24', 'weekend', 'Jati Mulya'),
(2, 1, '06:44', 'weekend', 'Jati Mulya'),
(2, 1, '07:04', 'weekend', 'Jati Mulya'),
(2, 1, '07:24', 'weekend', 'Jati Mulya'),
(2, 1, '07:44', 'weekend', 'Jati Mulya'),
(2, 1, '08:04', 'weekend', 'Jati Mulya'),
(2, 1, '08:24', 'weekend', 'Jati Mulya'),
(2, 1, '08:44', 'weekend', 'Jati Mulya'),
(2, 1, '09:04', 'weekend', 'Jati Mulya'),
(2, 1, '09:24', 'weekend', 'Jati Mulya'),
(2, 1, '09:44', 'weekend', 'Jati Mulya'),
(2, 1, '10:04', 'weekend', 'Jati Mulya'),
(2, 1, '10:24', 'weekend', 'Jati Mulya'),
(2, 1, '10:44', 'weekend', 'Jati Mulya'),
(2, 1, '11:04', 'weekend', 'Jati Mulya'),
(2, 1, '11:24', 'weekend', 'Jati Mulya'),
(2, 1, '11:44', 'weekend', 'Jati Mulya'),
(2, 1, '12:04', 'weekend', 'Jati Mulya'),
(2, 1, '12:24', 'weekend', 'Jati Mulya'),
(2, 1, '12:44', 'weekend', 'Jati Mulya'),
(2, 1, '13:04', 'weekend', 'Jati Mulya'),
(2, 1, '13:24', 'weekend', 'Jati Mulya'),
(2, 1, '13:44', 'weekend', 'Jati Mulya'),
(2, 1, '14:04', 'weekend', 'Jati Mulya'),
(2, 1, '14:24', 'weekend', 'Jati Mulya'),
(2, 1, '14:44', 'weekend', 'Jati Mulya'),
(2, 1, '15:04', 'weekend', 'Jati Mulya'),
(2, 1, '15:24', 'weekend', 'Jati Mulya'),
(2, 1, '15:44', 'weekend', 'Jati Mulya'),
(2, 1, '16:04', 'weekend', 'Jati Mulya'),
(2, 1, '16:24', 'weekend', 'Jati Mulya'),
(2, 1, '16:44', 'weekend', 'Jati Mulya'),
(2, 1, '17:04', 'weekend', 'Jati Mulya'),
(2, 1, '17:24', 'weekend', 'Jati Mulya'),
(2, 1, '17:44', 'weekend', 'Jati Mulya'),
(2, 1, '18:04', 'weekend', 'Jati Mulya'),
(2, 1, '18:24', 'weekend', 'Jati Mulya'),
(2, 1, '18:44', 'weekend', 'Jati Mulya');

-- Weekday schedules: Dukuh Atas → Jati Mulya (reverse)
INSERT INTO schedules (route_id, station_id, departure_time, schedule_type, direction) VALUES
(2, 1, '05:00', 'weekday', 'Jati Mulya'),
(2, 1, '05:15', 'weekday', 'Jati Mulya'),
(2, 1, '05:30', 'weekday', 'Jati Mulya'),
(2, 1, '05:45', 'weekday', 'Jati Mulya'),
(2, 1, '06:00', 'weekday', 'Jati Mulya'),
(2, 1, '06:08', 'weekday', 'Jati Mulya'),
(2, 1, '06:15', 'weekday', 'Jati Mulya'),
(2, 1, '06:22', 'weekday', 'Jati Mulya'),
(2, 1, '06:30', 'weekday', 'Jati Mulya'),
(2, 1, '06:38', 'weekday', 'Jati Mulya'),
(2, 1, '06:45', 'weekday', 'Jati Mulya'),
(2, 1, '06:52', 'weekday', 'Jati Mulya'),
(2, 1, '07:00', 'weekday', 'Jati Mulya'),
(2, 1, '07:08', 'weekday', 'Jati Mulya'),
(2, 1, '07:15', 'weekday', 'Jati Mulya'),
(2, 1, '07:22', 'weekday', 'Jati Mulya'),
(2, 1, '07:30', 'weekday', 'Jati Mulya'),
(2, 1, '07:38', 'weekday', 'Jati Mulya'),
(2, 1, '07:45', 'weekday', 'Jati Mulya'),
(2, 1, '07:52', 'weekday', 'Jati Mulya'),
(2, 1, '08:00', 'weekday', 'Jati Mulya'),
(2, 1, '08:08', 'weekday', 'Jati Mulya'),
(2, 1, '08:15', 'weekday', 'Jati Mulya'),
(2, 1, '08:22', 'weekday', 'Jati Mulya'),
(2, 1, '08:30', 'weekday', 'Jati Mulya'),
-- Midday
(2, 1, '09:00', 'weekday', 'Jati Mulya'),
(2, 1, '09:20', 'weekday', 'Jati Mulya'),
(2, 1, '09:40', 'weekday', 'Jati Mulya'),
(2, 1, '10:00', 'weekday', 'Jati Mulya'),
(2, 1, '10:20', 'weekday', 'Jati Mulya'),
(2, 1, '10:40', 'weekday', 'Jati Mulya'),
(2, 1, '11:00', 'weekday', 'Jati Mulya'),
(2, 1, '11:20', 'weekday', 'Jati Mulya'),
(2, 1, '11:40', 'weekday', 'Jati Mulya'),
(2, 1, '12:00', 'weekday', 'Jati Mulya'),
(2, 1, '12:20', 'weekday', 'Jati Mulya'),
(2, 1, '12:40', 'weekday', 'Jati Mulya'),
(2, 1, '13:00', 'weekday', 'Jati Mulya'),
(2, 1, '13:20', 'weekday', 'Jati Mulya'),
(2, 1, '13:40', 'weekday', 'Jati Mulya'),
(2, 1, '14:00', 'weekday', 'Jati Mulya'),
(2, 1, '14:20', 'weekday', 'Jati Mulya'),
(2, 1, '14:40', 'weekday', 'Jati Mulya'),
(2, 1, '15:00', 'weekday', 'Jati Mulya'),
(2, 1, '15:20', 'weekday', 'Jati Mulya'),
(2, 1, '15:40', 'weekday', 'Jati Mulya'),
-- Evening peak
(2, 1, '16:00', 'weekday', 'Jati Mulya'),
(2, 1, '16:08', 'weekday', 'Jati Mulya'),
(2, 1, '16:15', 'weekday', 'Jati Mulya'),
(2, 1, '16:22', 'weekday', 'Jati Mulya'),
(2, 1, '16:30', 'weekday', 'Jati Mulya'),
(2, 1, '16:38', 'weekday', 'Jati Mulya'),
(2, 1, '16:45', 'weekday', 'Jati Mulya'),
(2, 1, '16:52', 'weekday', 'Jati Mulya'),
(2, 1, '17:00', 'weekday', 'Jati Mulya'),
(2, 1, '17:08', 'weekday', 'Jati Mulya'),
(2, 1, '17:15', 'weekday', 'Jati Mulya'),
(2, 1, '17:22', 'weekday', 'Jati Mulya'),
(2, 1, '17:30', 'weekday', 'Jati Mulya'),
(2, 1, '17:38', 'weekday', 'Jati Mulya'),
(2, 1, '17:45', 'weekday', 'Jati Mulya'),
(2, 1, '17:52', 'weekday', 'Jati Mulya'),
(2, 1, '18:00', 'weekday', 'Jati Mulya'),
(2, 1, '18:08', 'weekday', 'Jati Mulya'),
(2, 1, '18:15', 'weekday', 'Jati Mulya'),
(2, 1, '18:22', 'weekday', 'Jati Mulya'),
(2, 1, '18:30', 'weekday', 'Jati Mulya'),
-- Night
(2, 1, '19:00', 'weekday', 'Jati Mulya'),
(2, 1, '19:30', 'weekday', 'Jati Mulya'),
(2, 1, '20:00', 'weekday', 'Jati Mulya'),
(2, 1, '20:30', 'weekday', 'Jati Mulya'),
(2, 1, '21:00', 'weekday', 'Jati Mulya'),
(2, 1, '21:30', 'weekday', 'Jati Mulya'),
(2, 1, '22:00', 'weekday', 'Jati Mulya');
