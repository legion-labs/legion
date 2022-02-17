const cookieDomain = window.location.hostname;

/**
 * Fast cookie lookup function
 * @param name Cookie name
 * @returns The cookie value or `null`
 */
export function getCookie(name: string) {
  const parts = document.cookie.split(/[;=]/);

  for (let i = 0; i < parts.length - 1; i += 2) {
    if (parts[i].trim() === name) {
      const value = parts[i + 1];

      return value && value.trim();
    }
  }

  return null;
}

/**
 * Safely set a cookie in the browser
 * @param name Cookie name
 * @param value Cookie value (must be serializable)
 * @param maxAge Cookie duration (in seconds)
 */
export function setCookie(
  name: string,
  value: { toString(): string },
  maxAge?: number
) {
  document.cookie = `${name}=${value.toString()};domain=${cookieDomain};path=/;max-age=${maxAge};samesite=strict;secure;`;
}
