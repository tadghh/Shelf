export function isValidDirectoryPath(path) {
    if (typeof path !== 'string') {
        return false;
    }

    // Check for Unix-like path
    if (path.startsWith('/')) {
        return true;
    }

    // Check for Windows path
    if (/^[a-zA-Z]:\\/.test(path)) {
        return true;
    }

    return false;
}