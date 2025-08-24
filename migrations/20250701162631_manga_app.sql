-- Add migration script here

-- Custom ENUM type for processing_status
CREATE TYPE series_status AS ENUM (
    'Pending',
    'Processing',
    'Available',
    'Ongoing',
    'Completed',
    'Hiatus',
    'Discontinued',
    'Error',
    'Pending Deletion',
    'Deleting',
    'Deletion Failed'
    );

-- Table to store general manga, manhwa, manhua, webtoon, comic series information
CREATE TABLE IF NOT EXISTS series
(
    id                            SERIAL PRIMARY KEY,
    title                         TEXT          NOT NULL,
    original_title                TEXT,
    description                   TEXT          NOT NULL,
    cover_image_url               TEXT          NOT NULL,
    current_source_url            TEXT          NOT NULL,
    source_website_host           TEXT          NOT NULL,
    views_count                   INTEGER       NOT NULL DEFAULT 0,
    bookmarks_count               INTEGER       NOT NULL DEFAULT 0,
    last_chapter_found_in_storage REAL                   DEFAULT 0,
    processing_status             series_status NOT NULL DEFAULT 'Pending',
    check_interval_minutes        INTEGER       NOT NULL DEFAULT 60,
    last_checked_at               TIMESTAMPTZ,
    next_checked_at               TIMESTAMPTZ,
    created_at                    TIMESTAMPTZ   NOT NULL DEFAULT NOW(),
    updated_at                    TIMESTAMPTZ   NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_series_title ON series (title);
CREATE INDEX IF NOT EXISTS idx_series_check ON series (next_checked_at, processing_status);

-- Table to store chapter information
CREATE TABLE IF NOT EXISTS series_chapters
(
    id             SERIAL PRIMARY KEY,
    series_id      INTEGER     NOT NULL REFERENCES series (id) ON DELETE CASCADE,
    chapter_number REAL        NOT NULL,        -- Support chapter numbers like 1, 2, 2.5, 3, etc.
    title          TEXT,                        -- Chapter title (can be NULL)
    source_url     TEXT        NOT NULL UNIQUE, -- URL of the chapter source
    created_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chapters_series_id ON series_chapters (series_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_chapters_series_number ON series_chapters (series_id, chapter_number);

-- Table to store chapter image information
CREATE TABLE IF NOT EXISTS chapter_images
(
    id          SERIAL PRIMARY KEY,
    chapter_id  INTEGER     NOT NULL REFERENCES series_chapters (id) ON DELETE CASCADE, -- Foreign key to chapters.id table
    image_order INTEGER     NOT NULL,                                                   -- Order of images in the chapter (e.g., 1, 2, 3, etc.)
    image_url   TEXT        NOT NULL,                                                   -- URL of the image R2/CDN
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_chapter_images_chapter_id ON chapter_images (chapter_id);
CREATE INDEX IF NOT EXISTS idx_chapter_images_order ON chapter_images (chapter_id, image_order);

-- Table to store Categories/Tags
CREATE TABLE IF NOT EXISTS categories
(
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS series_categories
(
    series_id   INTEGER NOT NULL REFERENCES series (id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES categories (id) ON DELETE CASCADE,
    PRIMARY KEY (series_id, category_id) -- Prevents duplicate entries
);

CREATE INDEX IF NOT EXISTS idx_series_categories_category_id ON series_categories (category_id);

-- Table to store Author/Artist information
CREATE TABLE IF NOT EXISTS authors
(
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS series_authors
(
    series_id INTEGER NOT NULL REFERENCES series (id) ON DELETE CASCADE,
    author_id INTEGER NOT NULL REFERENCES authors (id) ON DELETE CASCADE,
    PRIMARY KEY (series_id, author_id) -- Prevents duplicate entries
);

CREATE INDEX IF NOT EXISTS idx_series_authors_author_id ON series_authors (author_id);

-- Table to store roles
CREATE TABLE IF NOT EXISTS roles
(
    id        SERIAL PRIMARY KEY,
    role_name TEXT NOT NULL UNIQUE
);

INSERT INTO roles (role_name)
VALUES ('user'),
       ('admin'),
       ('moderator');


-- Table to store user information
CREATE TABLE IF NOT EXISTS users
(
    id            SERIAL PRIMARY KEY,
    username      TEXT        NOT NULL UNIQUE,
    email         TEXT        NOT NULL UNIQUE,
    password_hash TEXT        NOT NULL,
    role_id       INTEGER     NOT NULL, -- User role (1 = 'user', 2 = 'admin', 3 = 'moderator')
    is_active     BOOLEAN              DEFAULT TRUE,
    last_login_at TIMESTAMPTZ,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Foreign key constraint to link to the roles table
    CONSTRAINT fk_user_role
        FOREIGN KEY (role_id)
            REFERENCES roles (id)
            ON UPDATE CASCADE
            ON DELETE RESTRICT
);

-- Table to store user profile information
CREATE TABLE IF NOT EXISTS user_profiles
(
    id           SERIAL PRIMARY KEY,
    user_id      INTEGER NOT NULL UNIQUE REFERENCES users (id) ON DELETE CASCADE,
    display_name TEXT,
    avatar_url   TEXT
);

-- Table to store user bookmarks
CREATE TABLE IF NOT EXISTS user_bookmarks
(
    id            SERIAL PRIMARY KEY,
    user_id       INTEGER NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    series_id     INTEGER NOT NULL REFERENCES series (id) ON DELETE CASCADE,
    bookmarked_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_bookmarks_unique ON user_bookmarks (user_id, series_id);

-- Table to store password reset tokens
CREATE TABLE IF NOT EXISTS password_reset_tokens
(
    id         SERIAL PRIMARY KEY,
    user_id    INTEGER     NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    token      TEXT        NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token ON password_reset_tokens (token);
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_user_id ON password_reset_tokens (user_id);

-- Table to log each view for a series
CREATE TABLE IF NOT EXISTS series_view_log
(
    id        BIGSERIAL PRIMARY KEY,
    series_id INTEGER     NOT NULL REFERENCES series (id) ON DELETE CASCADE,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_series_view_log_series ON series_view_log (series_id);
CREATE INDEX IF NOT EXISTS idx_series_view_log_timed ON series_view_log (viewed_at DESC);