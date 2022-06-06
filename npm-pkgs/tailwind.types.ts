type TailwindParameters = {
  opacityVariable: string;
};

export type TailwindTransform = {
  withOpacity: (
    varName: string,
    defaultOpacityVariable?: string
  ) => (p: TailwindParameters) => string;
};
