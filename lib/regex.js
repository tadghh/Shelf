export function isValidDirectoryPath(path) {
    if (typeof path !== 'string') {
        return false;
    }

    // Check for Unix-like path
    if (path.startsWith('/')) {
        return true;
    }

    return (/^[a-zA-Z]:\\/.test(path));
}