class_name AddMenu extends PopupMenu

static var ENTRIES: Array = [
	["Geometry", "Sphere", SceneGeometry, SceneGeometry.Kind.SPHERE],
	["Geometry", "Cube", SceneGeometry, SceneGeometry.Kind.CUBE],
	["Geometry", "Triangle", SceneGeometry, SceneGeometry.Kind.TRIANGLE],
	["Geometry", "Mesh", SceneGeometry, SceneGeometry.Kind.MESH],
	["Light", "Point", SceneLight, SceneLight.Kind.POINT],
	["Light", "Directional", SceneLight, SceneLight.Kind.DIRECTIONAL],
	["Light", "Spot", SceneLight, SceneLight.Kind.SPOT],
	["Light", "Environment", SceneLight, SceneLight.Kind.ENVIRONMENT],
	["Camera", "Perspective", SceneCamera, SceneCamera.Kind.PERSPECTIVE],
	["Camera", "Orthographic", SceneCamera, SceneCamera.Kind.ORTHOGRAPHIC],
	["Misc", "Group", SceneObject, -1],
]

signal add_requested(node: SceneObject, node_name: String)

func _ready() -> void:
	id_pressed.connect(_on_id_pressed)

func open_below(button: Button) -> void:
	_build()

	var corner := button.get_global_rect().end

	popup_on_parent(Rect2i(Vector2i(corner)
		- Vector2i(int(size.x), 0), Vector2i(1, 1)))

func _build() -> void:
	clear()

	var group := ""

	for id in ENTRIES.size():
		var entry: Array = ENTRIES[id]

		if entry[0] != group:
			group = entry[0]
			add_item(group, -2)
			set_item_disabled(item_count - 1, true)

		add_item(entry[1], id)

	reset_size()

func _on_id_pressed(id: int) -> void:
	var entry: Array = ENTRIES[id]
	var node: SceneObject = entry[2].new()

	if entry[3] >= 0:
		node.kind = entry[3]

	add_requested.emit(node, entry[1])
