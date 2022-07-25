Object(
    ObjectType {
        name: "InlinesDependsOnPrimitives",
        id: TypeId {
            t: 11215793871398625876,
        },
        fields: [
            ObjectField {
                name: "d",
                ty: Object(
                    ObjectType {
                        name: "DependsOnPrimitives",
                        id: TypeId {
                            t: 2852309381660701282,
                        },
                        fields: [
                            ObjectField {
                                name: "p",
                                ty: Reference(
                                    "Primitives",
                                ),
                                optional: false,
                            },
                        ],
                        tag: None,
                    },
                ),
                optional: false,
            },
        ],
        tag: None,
    },
)