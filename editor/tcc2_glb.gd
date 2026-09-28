# Reads a .glb once and hands out what the editor references into it: a mesh
# primitive or an image. Knows nothing about nodes, so Tcc2Import can use it.
class_name Tcc2Glb

static var _cache := {}

static func state(path: String) -> GLTFState:
	if not _cache.has(path):
		var doc := GLTFDocument.new()
		var parsed := GLTFState.new()

		if doc.append_from_file(path, parsed) != OK:
			push_error("could not read %s" % path)
			parsed = null

		_cache[path] = parsed

	return _cache[path]

static func images(path: String) -> Array:
	var parsed := state(path)
	return parsed.get_images() if parsed != null else []
	
static func surface(path: String, mesh: int, primitive: int) -> Mesh:
	var parsed := state(path)

	if parsed == null or mesh < 0 or mesh >= parsed.get_meshes().size():
		return null

	var importer: ImporterMesh = parsed.get_meshes()[mesh].mesh

	if primitive < 0 or primitive >= importer.get_surface_count():
		return null

	var built := ArrayMesh.new()
	built.add_surface_from_arrays(
		importer.get_surface_primitive_type(primitive),
		importer.get_surface_arrays(primitive)
	)

	return built
