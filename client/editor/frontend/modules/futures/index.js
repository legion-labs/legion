export function debounce(func, wait, immediate) {
  let timeout;

  return function () {
    const context = this;
    const args = arguments;
    const later = function () {
      timeout = null;
      if (!immediate) func.apply(context, args);
    };
    const callNow = immediate && !timeout;
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
    if (callNow) func.apply(context, args);
  };
}

export function retryForever(fn) {
  return retry(-1, fn);
}

export function retry(maxRetries, fn) {
  return fn().catch(function (err) {
    if (maxRetries === 0) {
      throw err;
    }

    if (maxRetries > 0) {
      maxRetries--;
    }

    return retry(maxRetries, fn);
  });
}
