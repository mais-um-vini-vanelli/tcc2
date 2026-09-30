class_name Tcc2Assets

static var _documents := {}
static var _images := {}

static func state(path: String) -> GLTFState:
	if not _documents.has(path):
		var doc := GLTFDocument.new()
		var parsed := GLTFState.new()

		if doc.append_from_file(path, parsed) != OK:
			push_error("could not read %s" % path)
			parsed = null

		_documents[path] = parsed

	return _documents[path]

static func images(path: String) -> Array:
	if path.get_extension().to_lower() == "glb":
		var parsed := state(path)
		return parsed.get_images() if parsed != null else []

	if not _images.has(path):
		var picture := Image.load_from_file(path)

		if picture == null:
			push_error("could not read %s" % path)
			_images[path] = []
		else:
			_images[path] = [ImageTexture.create_from_image(picture)]

	return _images[path]

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
