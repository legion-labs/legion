export function bytes_out(x) {
    return Buffer(JSON.stringify(x)).toString('base64');
}

export function bytes_in(x) {
    return JSON.parse(Buffer.from(x, 'base64'));
}
