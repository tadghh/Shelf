export function isValidDirectoryPath(path) {
    // Regular expression to validate directory paths for both Unix-like and Windows
    const pathPattern = /^(\/[^<>:"|?*]*)|(?:[a-zA-Z]:(\\[^<>:"|?*]*)*)$/;
    return pathPattern.test(path);
}