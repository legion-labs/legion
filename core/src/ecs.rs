type EntityIdentifier = u64;
const INVALID_ID: EntityIdentifier = 0u;

struct Entity {
    id: EntityIdentifier,
}

trait Component {
}