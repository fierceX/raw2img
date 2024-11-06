-- 用户表
CREATE TABLE `users` (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    password TEXT NOT NULL,
    wb bool NOT NULL,
    half_size bool NOT NULL,
    quality BIGINT NOT NULL,
    lut_id BIGINT,
    UNIQUE(email)
);

-- 存储表
CREATE TABLE storages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    storage_name TEXT NOT NULL,
    storage_path TEXT,
    storage_type TEXT NOT NULL,
    storage_url TEXT,
    ACCESS_KEY TEXT,
    SECRET_KEY TEXT,
    bucket_name TEXT,
    added_time DATETIME NOT NULL,
    storage_usage TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id),
    UNIQUE(storage_name)
);


-- 路径表
CREATE TABLE paths (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    storage_id INTEGER NOT NULL,
    path TEXT NOT NULL,
    FOREIGN KEY (storage_id) REFERENCES storage(id),
    UNIQUE(storage_id,path)
);

-- 图片信息表

CREATE TABLE images (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    path_id INTEGER NOT NULL,
    cache_id INTEGER,
    file_name TEXT NOT NULL,
    cache_file_name TEXT ,
    scan_time DATETIME NOT NULL,
    shooting_time DATETIME NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type TEXT NOT NULL,
    exif TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (path_id) REFERENCES path(id),
    UNIQUE(path_id,file_name)
);

CREATE TABLE luts(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    storage_id INTEGER NOT NULL,
    lut_name TEXT NOT NULL,
    comment TEXT NOT NULL,
    UNIQUE(storage_id,lut_name)
);

-- 图片完整路径视图表
DROP VIEW IF EXISTS `images_view`;
CREATE VIEW images_view AS
SELECT 
    images.id,images.user_id,images.file_name,images.cache_file_name,images.scan_time,images.shooting_time,images.file_size,images.mime_type,images.exif,
    "/api/img/" || storage_original.storage_url || paths_original.path || "/" || images.file_name AS original_url,
    case storage_cached.storage_type
    when "local" then "/api/img/" || storage_cached.storage_url || paths_cached.path || "/" || images.cache_file_name
    else storage_cached.storage_url || paths_cached.path || "/" || images.cache_file_name
    end AS cached_url
FROM 
    images
LEFT JOIN 
    paths AS paths_original ON images.path_id = paths_original.id
LEFT JOIN 
    paths AS paths_cached ON images.cache_id = paths_cached.id
LEFT JOIN 
    storages AS storage_original ON paths_original.storage_id = storage_original.id
LEFT JOIN 
    storages AS storage_cached ON paths_cached.storage_id = storage_cached.id;
