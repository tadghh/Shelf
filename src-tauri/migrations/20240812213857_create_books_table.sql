-- Add migration script here
CREATE TABLE books (
    id INTEGER PRIMARY KEY AUTOINCREMENT,  -- Unique identifier for each book
    cover_location TEXT NOT NULL,           -- Path or URL to the book cover
    book_location TEXT NOT NULL,            -- Path or URL to the book content
    title TEXT NOT NULL                     -- Title of the book
);