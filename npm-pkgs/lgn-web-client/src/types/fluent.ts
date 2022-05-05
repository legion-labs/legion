import type { FluentVariable } from "@fluent/bundle";

export type FluentBase = Record<
  string,
  { attributes: string | null; variables: string | null }
>;

export type FluentBaseVariablesOnly = Record<
  string,
  Pick<FluentBase[string], "variables">
>;

export type FluentBaseAttributesOnly = Record<
  string,
  Pick<FluentBase[string], "attributes">
>;

export type ResolveFluentArguments<
  Fluent extends FluentBase,
  Id extends keyof Fluent
> = Fluent[Id]["attributes"] extends string
  ? Fluent[Id]["variables"] extends string
    ? [
        id: Id,
        attributes: Fluent[Id]["attributes"],
        variables: {
          [Key in Fluent[Id]["variables"]]: FluentVariable;
        }
      ]
    : [id: Id, attributes: Fluent[Id]["attributes"]]
  : Fluent[Id]["variables"] extends string
  ? [
      id: Id,
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      }
    ]
  : [id: Id];

export type ResolveFluentRecord<
  Fluent extends FluentBase,
  Id extends keyof Fluent
> = Fluent[Id]["attributes"] extends string
  ? Fluent[Id]["variables"] extends string
    ? {
        id: Id;
        attributes: Fluent[Id]["attributes"];
        variables: {
          [Key in Fluent[Id]["variables"]]: FluentVariable;
        };
      }
    : { id: Id; attributes: Fluent[Id]["attributes"] }
  : Fluent[Id]["variables"] extends string
  ? {
      id: Id;
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      };
    }
  : { id: Id };

export type ResolveFluentArgumentsVariablesOnly<
  Fluent extends FluentBaseVariablesOnly,
  Id extends keyof Fluent
> = Fluent[Id]["variables"] extends string
  ? [
      id: Id,
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      }
    ]
  : [id: Id];

export type ResolveFluentRecordVariablesOnly<
  Fluent extends FluentBaseVariablesOnly,
  Id extends keyof Fluent
> = Fluent[Id]["variables"] extends string
  ? {
      id: Id;
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      };
    }
  : { id: Id };

export type ResolveFluentArgumentsAttributesOnly<
  Fluent extends FluentBaseAttributesOnly,
  Id extends keyof Fluent
> = Fluent[Id]["attributes"] extends string
  ? [
      id: Id,
      attributes: {
        [Key in Fluent[Id]["attributes"]]: FluentVariable;
      }
    ]
  : [id: Id];

export type ResolveFluentRecordAttributesOnly<
  Fluent extends FluentBaseAttributesOnly,
  Id extends keyof Fluent
> = Fluent[Id]["attributes"] extends string
  ? {
      id: Id;
      attributes: {
        [Key in Fluent[Id]["attributes"]]: FluentVariable;
      };
    }
  : { id: Id };
