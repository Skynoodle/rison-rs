Rison is a data serialization format based on JSON, optimized for
compactness in URIs.

The format supported by this implementation is based on the documentation
and implementations found below:
- <https://github.com/Nanonid/rison>
- <https://github.com/w33ble/rison-node>

The deserializer implementation is broadly inspired by the existing
`serde_json` library which provides a `serde` serializer and
deserializer for the standard JSON format.