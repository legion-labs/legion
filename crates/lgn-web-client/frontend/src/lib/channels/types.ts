export type Subscriber<T> = (message: T) => void;

export type Destroyable = { destroy(): void };
