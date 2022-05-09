import type { FluentVariable } from "@fluent/bundle";

export type FluentBase = Record<
  string,
  { attributes?: string; variables?: string } | null
>;

export type FluentBaseVariablesOnly = Record<
  string,
  Pick<Exclude<FluentBase[string], null>, "variables"> | null
>;

export type FluentBaseAttributesOnly = Record<
  string,
  Pick<Exclude<FluentBase[string], null>, "attributes"> | null
>;

export type ResolveFluentArguments<
  Fluent extends FluentBase,
  Id extends keyof Fluent
> = Fluent[Id] extends { attributes: string; variables: string }
  ? [
      id: Id,
      attributes: Fluent[Id]["attributes"],
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      }
    ]
  : Fluent[Id] extends { attributes: string }
  ? [id: Id, attributes: Fluent[Id]["attributes"]]
  : Fluent[Id] extends { variables: string }
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
> = Fluent[Id] extends { attributes: string; variables: string }
  ? {
      id: Id;
      attributes: Fluent[Id]["attributes"];
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      };
    }
  : Fluent[Id] extends { attributes: string }
  ? { id: Id; attributes: Fluent[Id]["attributes"] }
  : Fluent[Id] extends { variables: string }
  ? {
      id: Id;
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      };
    }
  : { id: Id };

export type ResolveFluentArgumentsVariablesOnly<
  Fluent extends FluentBase,
  Id extends keyof Fluent
> = Fluent[Id] extends { variables: string }
  ? [
      id: Id,
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      }
    ]
  : [id: Id];

export type ResolveFluentRecordVariablesOnly<
  Fluent extends FluentBase,
  Id extends keyof Fluent
> = Fluent[Id] extends { variables: string }
  ? {
      id: Id;
      variables: {
        [Key in Fluent[Id]["variables"]]: FluentVariable;
      };
    }
  : { id: Id };

export type ResolveFluentArgumentsAttributesOnly<
  Fluent extends FluentBase,
  Id extends keyof Fluent
> = Fluent[Id] extends { attributes: string }
  ? [id: Id, attributes: Fluent[Id]["attributes"]]
  : [id: Id];

export type ResolveFluentRecordAttributesOnly<
  Fluent extends FluentBase,
  Id extends keyof Fluent
> = Fluent[Id] extends { attributes: string }
  ? {
      id: Id;
      attributes: Fluent[Id]["attributes"];
    }
  : { id: Id };
