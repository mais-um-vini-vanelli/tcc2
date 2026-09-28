extends Control

@onready var hierarchy: Tree = $Hierarchy/VBoxContainer/MarginContainer/Tree
@onready var sub_viewport_container := $SubViewportContainer
@onready var scene_view: SubViewport = $SubViewportContainer/SceneView
@onready var v_box_container := $Properties/VBoxContainer/MarginContainer/ScrollContainer/VBoxContainer
@onready var popup_menu := $PopupMenu

var root: Tcc2Scene
var _items := {}
var _selected: Tcc2Node

func _ready() -> void:
	root = Tcc2Scene.new()
	root.name = "Scene"
	scene_view.add_child(root)

	var camera := Tcc2PerspectiveCamera.new()
	camera.name = "Camera"
	camera.position = Vector3(0, 1, 4)
	root.add_child(camera)
	camera.rotation_degrees = Vector3(-14, 0, 0)
	camera.owner = root

	var light := Tcc2DirectionalLight.new()
	light.name = "Light"
	light.position = Vector3(-2, 3, 2)
	root.add_child(light)
	light.rotation_degrees = Vector3(-45, -35, 0)
	light.owner = root

	var sphere := Tcc2Sphere.new()
	sphere.name = "Sphere"
	root.add_child(sphere)
	sphere.owner = root

	hierarchy.allow_rmb_select = true
	rebuild_hierarchy()
	hierarchy.item_selected.connect(_on_selected)

	sub_viewport_container.tcc2_node_selected.connect(_on_sub_viewport_container_tcc2_node_selected)
	hierarchy.item_mouse_selected.connect(_on_mouse_select)
	popup_menu.id_pressed.connect(_on_popup_menu_id_pressed)
	get_window().files_dropped.connect(_on_files_dropped)
	sub_viewport_container.gizmo_moved.connect(v_box_container.refresh_values)

func rebuild_hierarchy() -> void:
	hierarchy.clear()
	_items.clear()
	add_item(root, null)

func add_item(node: Tcc2Node, parent_item: TreeItem) -> void:
	var item := hierarchy.create_item(parent_item)
	item.set_text(0, node.name)
	item.set_metadata(0, node)
	_items[node] = item

	for child in node.get_children():
		if child is Tcc2Node:
			add_item(child, item)

func _on_sub_viewport_container_tcc2_node_selected(node: Tcc2Node) -> void:
	if node == null:
		hierarchy.deselect_all()
		_select(null)
	else:
		var item: TreeItem = _items[node]
		item.select(0)
		_select(node)
		hierarchy.scroll_to_item(item)

func _select(node: Tcc2Node) -> void:
	if _selected != null and is_instance_valid(_selected):
		_selected.set_selected(false)

	_selected = node

	if node != null:
		node.set_selected(true)

	sub_viewport_container.set_target(node)
	v_box_container.show_node(node)

func _on_selected() -> void:
	_select(hierarchy.get_selected().get_metadata(0))

func _unhandled_key_input(event: InputEvent) -> void:
	var key := event as InputEventKey

	if key == null or not key.pressed or key.echo:
		return

	if key.keycode == KEY_DELETE and _selected != null and _selected != root:
		_remove(_selected)

func _remove(node: Tcc2Node) -> void:
	_select(null)

	node.get_parent().remove_child(node)
	node.queue_free()

	rebuild_hierarchy()

enum {
	ADD_SPHERE,
	ADD_CUBE,
	ADD_MESH,
	ADD_POINT_LIGHT,
	ADD_DIRECTIONAL_LIGHT,
	ADD_SPOT_LIGHT,
	ADD_AMBIENT_LIGHT,
	ADD_PERSPECTIVE_CAMERA,
	ADD_ORTHOGRAPHIC_CAMERA,
}

func _on_mouse_select(_pos: Vector2, button: int) -> void:
	if button == MOUSE_BUTTON_RIGHT:
		popup_menu.clear()
		popup_menu.add_item("Add sphere", ADD_SPHERE)
		popup_menu.add_item("Add cube", ADD_CUBE)
		popup_menu.add_item("Add mesh", ADD_MESH)

		popup_menu.add_separator()
		popup_menu.add_item("Add point light", ADD_POINT_LIGHT)
		popup_menu.add_item("Add directional light", ADD_DIRECTIONAL_LIGHT)
		popup_menu.add_item("Add spot light", ADD_SPOT_LIGHT)
		popup_menu.add_item("Add ambient light", ADD_AMBIENT_LIGHT)

		popup_menu.add_separator()
		popup_menu.add_item("Add perspective camera", ADD_PERSPECTIVE_CAMERA)
		popup_menu.add_item("Add orthographic camera", ADD_ORTHOGRAPHIC_CAMERA)

		popup_menu.reset_size()
		popup_menu.popup_on_parent(Rect2i(get_global_mouse_position(), Vector2i(1, 1)))

func _on_popup_menu_id_pressed(id: int) -> void:
	var target = hierarchy.get_selected().get_metadata(0)

	match id:
		ADD_SPHERE:
			_add(target, Tcc2Sphere.new(), "Sphere")

		ADD_CUBE:
			_add(target, Tcc2Cube.new(), "Cube")

		ADD_MESH:
			_add(target, Tcc2Mesh.new(), "Mesh")

		ADD_POINT_LIGHT:
			_add(target, Tcc2PointLight.new(), "PointLight")

		ADD_DIRECTIONAL_LIGHT:
			_add(target, Tcc2DirectionalLight.new(), "DirectionalLight")

		ADD_SPOT_LIGHT:
			_add(target, Tcc2SpotLight.new(), "SpotLight")

		ADD_AMBIENT_LIGHT:
			_add(target, Tcc2AmbientLight.new(), "AmbientLight")

		ADD_PERSPECTIVE_CAMERA:
			_add(target, Tcc2PerspectiveCamera.new(), "Camera")

		ADD_ORTHOGRAPHIC_CAMERA:
			_add(target, Tcc2OrthographicCamera.new(), "Camera")

	rebuild_hierarchy()

func _add(target: Tcc2Node, node: Tcc2Node, node_name: String) -> void:
	node.name = node_name
	target.add_child(node)
	_own(node)

func _own(node: Node) -> void:
	node.owner = root

	for child in node.get_children():
		_own(child)

func _on_files_dropped(files: PackedStringArray) -> void:
	for file in files:
		if file.get_extension().to_lower() != "glb":
			continue

		var group := Tcc2Import.glb(file, true)

		if group == null:
			continue

		root.add_child(group)
		_own(group)

	rebuild_hierarchy()
