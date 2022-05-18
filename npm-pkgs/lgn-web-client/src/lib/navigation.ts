import type { Page } from "@sveltejs/kit";

/**
 * Takes a `page` store provided by SvelteKit and the name of a search param to remove,
 * and returns a relative url with the search params updated accordingly.
 */
export function deleteSearchParam<Params extends Record<string, string>>(
  page: Page<Params>,
  name: string
) {
  const { pathname, searchParams } = page.url;

  const newSearchParams = new URLSearchParams(searchParams);

  newSearchParams.delete(name);

  const queryString = newSearchParams.toString();

  return `${pathname}${queryString.length ? `?${queryString}` : ""}`;
}

/**
 * Takes a `page` store provided by SvelteKit and an array of names of search params to remove,
 * and returns a relative url with the search params updated accordingly.
 */
export function deleteSearchParams<Params extends Record<string, string>>(
  page: Page<Params>,
  names: string[]
) {
  const { pathname, searchParams } = page.url;

  const newSearchParams = new URLSearchParams(searchParams);

  for (const name of names) {
    newSearchParams.delete(name);
  }

  const queryString = newSearchParams.toString();

  return `${pathname}${queryString.length ? `?${queryString}` : ""}`;
}

/**
 * Takes a `page` store provided by SvelteKit and the name/value pair of the search param to add,
 * and returns a relative url with the search params updated accordingly.
 */
export function setSearchParam<Params extends Record<string, string>>(
  page: Page<Params>,
  name: string,
  value: string
) {
  const { pathname, searchParams } = page.url;

  const newSearchParams = new URLSearchParams(searchParams);

  newSearchParams.set(name, value);

  const queryString = newSearchParams.toString();

  return `${pathname}${queryString.length ? `?${queryString}` : ""}`;
}

/**
 * Takes a `page` store provided by SvelteKit and a record search params to add,
 * and returns a relative url with the search params updated accordingly.
 */
export function setSearchParams<Params extends Record<string, string>>(
  page: Page<Params>,
  params: Record<string, string>
) {
  const { pathname, searchParams } = page.url;

  const newSearchParams = new URLSearchParams(searchParams);

  for (const [name, value] of Object.entries(params)) {
    newSearchParams.set(name, value);
  }

  const queryString = newSearchParams.toString();

  return `${pathname}${queryString.length ? `?${queryString}` : ""}`;
}

/**
 * Takes a `page` store provided by SvelteKit and an "update" function that will
 * receive the query params (as a plain record) and is expected to return the query params updated
 * as a plain object,
 * and returns a relative url with the search params updated accordingly.
 */
export function updateSearchParams<Params extends Record<string, string>>(
  page: Page<Params>,
  update: (params: Record<string, string>) => Record<string, string>
) {
  const { pathname, searchParams } = page.url;

  const newSearchParams = new URLSearchParams(
    update(Object.fromEntries(searchParams.entries()))
  );

  const queryString = newSearchParams.toString();

  return `${pathname}${queryString.length ? `?${queryString}` : ""}`;
}
