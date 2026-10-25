class_name SceneAssets extends RefCounted

enum Kind {IMAGE, MESH}

var _images: Array[Image] = []
var _image_labels: Array[String] = []
var _textures: Array[ImageTexture] = []
var _meshes: Array[Mesh] = []
var _mesh_labels: Array[String] = []

func image_count() -> int:
	return _images.size()

func mesh_count() -> int:
	return _meshes.size()

func add_image(image: Image, label: String) -> int:
	_images.append(image)
	_image_labels.append(label)
	_textures.append(null)

	return _images.size() - 1

func add_mesh(mesh: Mesh, label: String) -> int:
	_meshes.append(mesh)
	_mesh_labels.append(label)

	return _meshes.size() - 1

func image(index: int) -> Image:
	return _images[index] if index >= 0 and index < _images.size() else null

func mesh(index: int) -> Mesh:
	return _meshes[index] if index >= 0 and index < _meshes.size() else null

func texture(index: int) -> ImageTexture:
	if index < 0 or index >= _images.size():
		return null

	if _textures[index] == null:
		_textures[index] = ImageTexture.create_from_image(_images[index])

	return _textures[index]

func find_image(label: String) -> int:
	return _image_labels.find(label)

func find_mesh(label: String) -> int:
	return _mesh_labels.find(label)

func name(kind: Kind, index: int) -> String:
	var labels := _labels(kind)

	return labels[index] if index >= 0 and index < labels.size() else ""

static func label(path: String, entry: String) -> String:
	return "%s#%s" % [path.get_file(), entry]

## O arquivo de onde a entrada veio.
func file(kind: Kind, index: int) -> String:
	return _file_of(name(kind, index))

## O que distingue a entrada dentro do arquivo.
func entry(kind: Kind, index: int) -> String:
	var label := name(kind, index)
	var at := label.rfind("#")

	return label.substr(at + 1) if at >= 0 else label

## As entradas que vieram do mesmo arquivo que `index`, ela inclusa.
func siblings(kind: Kind, index: int) -> PackedInt32Array:
	var found := PackedInt32Array()
	var labels := _labels(kind)

	if index < 0 or index >= labels.size():
		return found

	var group := _file_of(labels[index])

	for i in labels.size():
		if _file_of(labels[i]) == group:
			found.append(i)

	return found

func _labels(kind: Kind) -> Array[String]:
	return _image_labels if kind == Kind.IMAGE else _mesh_labels

static func _file_of(label: String) -> String:
	var at := label.rfind("#")

	return label.substr(0, at) if at >= 0 else label
