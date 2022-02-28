import {
  buildDefaultPrimitiveProperty,
  buildGroupProperty,
  buildOptionProperty,
  buildVecProperty,
  extractOptionPType,
  extractVecPType,
  formatProperties,
  propertyIsBag,
  propertyIsBoolean,
  propertyIsColor,
  propertyIsComponent,
  propertyIsGroup,
  propertyIsNumber,
  propertyIsOption,
  propertyIsPrimitive,
  propertyIsQuat,
  propertyIsSpeed,
  propertyIsString,
  propertyIsResourcePathId,
  propertyIsEnum,
  propertyIsVec,
  propertyIsVec3,
  ptypeBelongsToPrimitive,
  ResourceProperty,
} from "@/lib/propertyGrid";
import propertiesResponse from "@/resources/propertiesResponse.json";

describe("formatProperties", () => {
  test("properly formats the properties received from the server", () => {
    expect(
      formatProperties(propertiesResponse as unknown as ResourceProperty[])
    ).toMatchSnapshot();
  });
});

describe("buildDefaultPrimitiveProperty", () => {
  test("builds a default primitive value from ptype `bool`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "bool")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `Speed`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "Speed")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `Color`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "Color")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `String`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "String")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `i32`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "i32")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `u32`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "u32")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `f32`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "f32")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `f64`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "f64")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `usize`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "usize")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `u8`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "u8")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `Vec3`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "Vec3")
    ).toMatchSnapshot();
  });

  test("builds a default primitive value from ptype `Quat`", () => {
    expect(
      buildDefaultPrimitiveProperty("My resource property", "Quat")
    ).toMatchSnapshot();
  });
});

describe("propertyIsBoolean", () => {
  test("returns `true` when the property's `ptype` === `bool`", () => {
    expect(
      propertyIsBoolean(
        buildDefaultPrimitiveProperty("My resource property", "bool")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `bool`", () => {
    expect(
      propertyIsBoolean(
        buildDefaultPrimitiveProperty("My resource property", "Color")
      )
    ).toBe(false);
  });
});

describe("propertyIsColor", () => {
  test("returns `true` when the property's `ptype` === `Color`", () => {
    expect(
      propertyIsColor(
        buildDefaultPrimitiveProperty("My resource property", "Color")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `Color`", () => {
    expect(
      propertyIsColor(
        buildDefaultPrimitiveProperty("My resource property", "String")
      )
    ).toBe(false);
  });
});

describe("propertyIsSpeed", () => {
  test("returns `true` when the property's `ptype` === `Speed`", () => {
    expect(
      propertyIsSpeed(
        buildDefaultPrimitiveProperty("My resource property", "Speed")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `Speed`", () => {
    expect(
      propertyIsSpeed(
        buildDefaultPrimitiveProperty("My resource property", "String")
      )
    ).toBe(false);
  });
});

describe("propertyIsString", () => {
  test("returns `true` when the property's `ptype` === `String`", () => {
    expect(
      propertyIsString(
        buildDefaultPrimitiveProperty("My resource property", "String")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `String`", () => {
    expect(
      propertyIsString(
        buildDefaultPrimitiveProperty("My resource property", "i32")
      )
    ).toBe(false);
  });
});

describe("propertyIsResourcePathId", () => {
  test("returns `true` when the property's `ptype` === `ResourcePathId`", () => {
    expect(
      propertyIsResourcePathId(
        buildDefaultPrimitiveProperty("My resource property", "ResourcePathId")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `ResourcePathId`", () => {
    expect(
      propertyIsResourcePathId(
        buildDefaultPrimitiveProperty("My resource property", "i32")
      )
    ).toBe(false);
  });
});

describe("propertyIsEnum", () => {
  test("returns `true` when the property's `ptype` starts with `_enum_:`", () => {
    expect(
      propertyIsEnum(
        buildDefaultPrimitiveProperty("My resource property", "_enum_:Test")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `_enum_:`", () => {
    expect(
      propertyIsEnum(
        buildDefaultPrimitiveProperty("My resource property", "i32")
      )
    ).toBe(false);
  });
});

describe("propertyIsNumber", () => {
  test("returns `true` when the property's `ptype` === `i32`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "i32")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `u32`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "u32")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `f32`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "f32")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `f64`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "f64")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `usize`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "usize")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `u8`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "u8")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `i32`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "Vec3")
      )
    ).toBe(false);
  });

  test("returns `false` when the property's `ptype` !== `u32`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "Vec3")
      )
    ).toBe(false);
  });

  test("returns `false` when the property's `ptype` !== `f32`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "Vec3")
      )
    ).toBe(false);
  });

  test("returns `false` when the property's `ptype` !== `f64`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "Vec3")
      )
    ).toBe(false);
  });

  test("returns `false` when the property's `ptype` !== `usize`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "Vec3")
      )
    ).toBe(false);
  });

  test("returns `false` when the property's `ptype` !== `u8`", () => {
    expect(
      propertyIsNumber(
        buildDefaultPrimitiveProperty("My resource property", "Vec3")
      )
    ).toBe(false);
  });
});

describe("propertyIsVec3", () => {
  test("returns `true` when the property's `ptype` === `Vec3`", () => {
    expect(
      propertyIsVec3(
        buildDefaultPrimitiveProperty("My resource property", "Vec3")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `Vec3`", () => {
    expect(
      propertyIsVec3(
        buildDefaultPrimitiveProperty("My resource property", "Quat")
      )
    ).toBe(false);
  });
});

describe("propertyIsQuat", () => {
  test("returns `true` when the property's `ptype` === `Quat`", () => {
    expect(
      propertyIsQuat(
        buildDefaultPrimitiveProperty("My resource property", "Quat")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `Quat`", () => {
    expect(
      propertyIsQuat(
        buildDefaultPrimitiveProperty("My resource property", "bool")
      )
    ).toBe(false);
  });
});

describe("extractOptionPType", () => {
  test("Extracts and return the inner ptype for Option property", () => {
    expect(
      extractOptionPType({
        attributes: {},
        ptype: "Option<String>",
        name: "My resource property",
        subProperties: [],
      })
    ).toBe("String");
  });

  test("returns `null` if the `ptype` doesn't belong to an Option property", () => {
    expect(
      extractOptionPType({
        attributes: {},
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        ptype: "Vec<String>" as any,
        name: "My resource property",
        subProperties: [],
      })
    ).toBe(null);
  });

  test("returns `null` if the `ptype` is invalid", () => {
    expect(
      extractOptionPType({
        attributes: {},
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        ptype: "Option<String" as any,
        name: "My resource property",
        subProperties: [],
      })
    ).toBe(null);
  });
});

describe("extractVecPType", () => {
  test("Extracts and return the inner ptype for Vec property", () => {
    expect(
      extractVecPType({
        attributes: {},
        ptype: "Vec<String>",
        name: "My resource property",
        subProperties: [],
      })
    ).toBe("String");
  });

  test("returns `null` if the `ptype` doesn't belong to a Vec property", () => {
    expect(
      extractVecPType({
        attributes: {},
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        ptype: "Option<String>" as any,
        name: "My resource property",
        subProperties: [],
      })
    ).toBe(null);
  });

  test("returns `null` if the `ptype` is invalid", () => {
    expect(
      extractVecPType({
        attributes: {},
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        ptype: "Vec<String" as any,
        name: "My resource property",
        subProperties: [],
      })
    ).toBe(null);
  });
});

describe("ptypeBelongsToPrimitive", () => {
  test("returns `true` when the `ptype` === `bool`", () => {
    expect(ptypeBelongsToPrimitive("bool")).toBe(true);
  });

  test("returns `true` when the `ptype` === `Speed`", () => {
    expect(ptypeBelongsToPrimitive("Speed")).toBe(true);
  });

  test("returns `true` when the `ptype` === `Color`", () => {
    expect(ptypeBelongsToPrimitive("Color")).toBe(true);
  });

  test("returns `true` when the `ptype` === `String`", () => {
    expect(ptypeBelongsToPrimitive("String")).toBe(true);
  });

  test("returns `true` when the `ptype` === `i32`", () => {
    expect(ptypeBelongsToPrimitive("i32")).toBe(true);
  });

  test("returns `true` when the `ptype` === `u32`", () => {
    expect(ptypeBelongsToPrimitive("u32")).toBe(true);
  });

  test("returns `true` when the `ptype` === `f32`", () => {
    expect(ptypeBelongsToPrimitive("f32")).toBe(true);
  });

  test("returns `true` when the `ptype` === `f64`", () => {
    expect(ptypeBelongsToPrimitive("f64")).toBe(true);
  });

  test("returns `true` when the `ptype` === `usize`", () => {
    expect(ptypeBelongsToPrimitive("usize")).toBe(true);
  });

  test("returns `true` when the `ptype` === `u8`", () => {
    expect(ptypeBelongsToPrimitive("u8")).toBe(true);
  });

  test("returns `true` when the `ptype` === `Vec3`", () => {
    expect(ptypeBelongsToPrimitive("Vec3")).toBe(true);
  });

  test("returns `true` when the `ptype` === `Quat`", () => {
    expect(ptypeBelongsToPrimitive("Quat")).toBe(true);
  });

  test("returns `false` when the `ptype` doesn't belong to a primitive", () => {
    expect(ptypeBelongsToPrimitive("Vec<String>")).toBe(false);
  });
});

describe("propertyIsPrimitive", () => {
  test("returns `true` when the property's `ptype` === `bool`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "bool")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `Speed`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "Speed")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `Color`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "Color")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `String`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "String")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `i32`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "i32")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `u32`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "u32")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `f32`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "f32")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `f64`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "f64")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `usize`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "usize")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `u8`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "u8")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `Vec3`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "Vec3")
      )
    ).toBe(true);
  });

  test("returns `true` when the property's `ptype` === `Quat`", () => {
    expect(
      propertyIsPrimitive(
        buildDefaultPrimitiveProperty("My resource property", "Quat")
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` doesn't belong to a primitive", () => {
    expect(
      propertyIsPrimitive({
        attributes: {},
        name: "My resource property",
        ptype: "Option<String>",
        subProperties: [],
      })
    ).toBe(false);
  });
});

describe("propertyIsOption", () => {
  test("returns `true` when the property's `ptype` matches `Option<.*>`", () => {
    expect(
      propertyIsOption(
        buildOptionProperty(
          "My resource property",
          buildDefaultPrimitiveProperty("My resource property", "Quat")
        )
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` doesn't match `Option<.*>`", () => {
    expect(
      propertyIsOption(
        buildVecProperty("My resource property", [
          buildDefaultPrimitiveProperty("[0]", "Quat"),
          buildDefaultPrimitiveProperty("[1]", "Quat"),
          buildDefaultPrimitiveProperty("[2]", "Quat"),
        ])
      )
    ).toBe(false);
  });

  test("returns `false` when the property is a primitive", () => {
    expect(
      propertyIsOption(
        buildDefaultPrimitiveProperty("My resource property", "Quat")
      )
    ).toBe(false);
  });
});

describe("propertyIsVec", () => {
  test("returns `true` when the property's `ptype` matches `Vec<.*>`", () => {
    expect(
      propertyIsVec(
        buildVecProperty("My resource property", [
          buildDefaultPrimitiveProperty("[0]", "Quat"),
          buildDefaultPrimitiveProperty("[1]", "Quat"),
          buildDefaultPrimitiveProperty("[2]", "Quat"),
        ])
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` doesn't match `Vec<.*>`", () => {
    expect(
      propertyIsVec(
        buildOptionProperty(
          "My resource property",
          buildDefaultPrimitiveProperty("My resource property", "Quat")
        )
      )
    ).toBe(false);
  });

  test("returns `false` when the property is a primitive", () => {
    expect(
      propertyIsVec(
        buildDefaultPrimitiveProperty("My resource property", "Quat")
      )
    ).toBe(false);
  });
});

describe("propertyIsGroup", () => {
  test("returns `true` when the property's `ptype` === `group`", () => {
    expect(
      propertyIsGroup(
        buildGroupProperty("My resource property", [
          buildDefaultPrimitiveProperty("My resource property", "Quat"),
        ])
      )
    ).toBe(true);
  });

  test("returns `false` when the property's `ptype` !== `group`", () => {
    expect(
      propertyIsGroup(
        buildOptionProperty(
          "My resource property",
          buildDefaultPrimitiveProperty("My resource property", "Quat")
        )
      )
    ).toBe(false);
  });

  test("returns `false` when the property is a primitive", () => {
    expect(
      propertyIsGroup(
        buildDefaultPrimitiveProperty("My resource property", "Quat")
      )
    ).toBe(false);
  });
});

describe("propertyIsComponent", () => {
  test("returns `true` when the property has an unknown `ptype` and it's assumed to be a Component", () => {
    expect(
      propertyIsComponent({
        attributes: {},
        name: "My resource property",
        ptype: "ComplexStruct",
        subProperties: [],
      })
    ).toBe(true);
  });

  test("returns `false` when the property is a primitive", () => {
    expect(
      propertyIsComponent(
        buildDefaultPrimitiveProperty("My resource property", "Quat")
      )
    ).toBe(false);
  });

  test("returns `false` when the property is an Option", () => {
    expect(
      propertyIsComponent(
        buildOptionProperty(
          "My resource property",
          buildDefaultPrimitiveProperty("My resource property", "Quat")
        )
      )
    ).toBe(false);
  });

  test("returns `false` when the property is a Vec", () => {
    expect(
      propertyIsComponent(
        buildVecProperty("My resource property", [
          buildDefaultPrimitiveProperty("[0]", "Quat"),
          buildDefaultPrimitiveProperty("[1]", "Quat"),
          buildDefaultPrimitiveProperty("[2]", "Quat"),
        ])
      )
    ).toBe(false);
  });

  test("returns `false` when the property is a group", () => {
    expect(
      propertyIsComponent(
        buildGroupProperty("My resource property", [
          buildDefaultPrimitiveProperty("My resource property", "Quat"),
        ])
      )
    ).toBe(false);
  });
});

describe("propertyIsBag", () => {
  test("returns `true` when the the property's `ptype` === `group`", () => {
    expect(
      propertyIsBag(
        buildGroupProperty("My resource property", [
          buildDefaultPrimitiveProperty("My resource property", "Quat"),
        ])
      )
    ).toBe(true);
  });

  test("returns `true` when the the property is a Component", () => {
    expect(
      propertyIsBag({
        attributes: {},
        name: "My resource property",
        ptype: "ComplexStruct",
        subProperties: [],
      })
    ).toBe(true);
  });

  test("returns `true` when the the property's `ptype` matches `Vec<*>`", () => {
    expect(
      propertyIsBag(
        buildVecProperty("My resource property", [
          buildDefaultPrimitiveProperty("My resource property", "Quat"),
        ])
      )
    ).toBe(true);
  });

  test("returns `true` when the the property's `ptype` matches `Option<*>` and the inner `ptype` is not a primitive's", () => {
    expect(
      propertyIsBag(
        buildOptionProperty("My resource property", {
          attributes: {},
          name: "My resource property",
          ptype: "ComplexStruct",
          subProperties: [],
        })
      )
    ).toBe(true);
  });

  test("returns `false` when the the property's `ptype` matches `Option<*>`and the inner `ptype` is a primitive's", () => {
    expect(
      propertyIsBag(
        buildOptionProperty(
          "My resource property",
          buildDefaultPrimitiveProperty("My resource property", "Quat")
        )
      )
    ).toBe(false);
  });

  test("returns `false` when the property is a primitive", () => {
    expect(
      propertyIsBag(
        buildDefaultPrimitiveProperty("My resource property", "Quat")
      )
    ).toBe(false);
  });
});
