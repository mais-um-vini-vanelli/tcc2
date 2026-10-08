extends Control

@onready var hierarchy: Tree = $Layout/Sidebar/Hierarchy/VBoxContainer/MarginContainer/Tree
@onready var save_button: Button = $Layout/Viewport/Overlay/VBoxContainer/Save
@onready var open_button: Button = $Layout/Viewport/Overlay/VBoxContainer/Open
@onready var sub_viewport_container := $Layout/Viewport/ViewportClip/SubViewportContainer
@onready var scene_view: SubViewport = $Layout/Viewport/ViewportClip/SubViewportContainer/SceneView
@onready var properties := $Layout/Sidebar/Properties/VBoxContainer/MarginContainer/ScrollContainer/Padding/VBoxContainer
@onready var popup_menu: PopupMenu = $PopupMenu
@onready var camera_pov: Button = $Layout/Viewport/Overlay/VBoxContainer/CameraPOV
@onready var pick_triangle: Button = $Layout/Viewport/Overlay/VBoxContainer/PickTriangle
@onready var pick_pixel: Button = $Layout/Viewport/Overlay/VBoxContainer/PickPixel

const CURSOR := preload("res://icons/Cursor.svg")

static var ICONS: Array = [
	[Tcc2Scene, preload("res://icons/Scene.svg"), Color("#d5dee8")],
	[Tcc2Light, preload("res://icons/Light.svg"), Color("#e6c94d")],
	[Tcc2Camera, preload("res://icons/Camera.svg"), Color("#d86481")],
	[Tcc2Geometry, preload("res://icons/Geometry.svg"), Color("#51cc8d")],
]

var root: Tcc2Scene
var save_dialog: FileDialog
var open_dialog: FileDialog
var _items := {}
var _selected: Tcc2Node

func _ready() -> void:
	var thin_divider = func(split: SplitContainer) -> void:
		# The default grabber prevents the divider from getting really thin.
		var image := Image.create_empty(1, 1, false, Image.FORMAT_RGBA8)
		split.add_theme_icon_override("grabber", ImageTexture.create_from_image(image))
		split.add_theme_constant_override("minimum_grab_thickness", 8)

	thin_divider.call($Layout)
	thin_divider.call($Layout/Sidebar)

	Input.set_custom_mouse_cursor(CURSOR, Input.CURSOR_ARROW, Vector2(6, 4))
	
	root = Tcc2Scene.new()
	root.name = "Scene"
	scene_view.add_child(root)

	var camera := Tcc2Camera.new()
	camera.kind = Tcc2Camera.Kind.PERSPECTIVE
	camera.name = "PerspectiveCamera"
	camera.position = Vector3(0, 1, 4)
	root.add_child(camera)
	camera.rotation_degrees = Vector3(-14, 0, 0)
	camera.owner = root

	var light := Tcc2Light.new()
	light.kind = Tcc2Light.Kind.DIRECTIONAL
	light.name = "DirectionalLight"
	light.position = Vector3(-2, 3, 2)
	root.add_child(light)
	light.rotation_degrees = Vector3(-45, -35, 0)
	light.owner = root

	var sphere := Tcc2Geometry.new()
	sphere.kind = Tcc2Geometry.Kind.SPHERE
	sphere.name = "Sphere"
	root.add_child(sphere)
	sphere.owner = root

	hierarchy.allow_rmb_select = true
	rebuild_hierarchy()
	hierarchy.item_selected.connect(_on_selected)
	_items[root].select(0)
	_select(root)

	sub_viewport_container.tcc2_node_selected.connect(_on_sub_viewport_container_tcc2_node_selected)
	hierarchy.item_mouse_selected.connect(_on_mouse_select)
	popup_menu.id_pressed.connect(_on_popup_menu_id_pressed)
	get_window().files_dropped.connect(_on_files_dropped)
	sub_viewport_container.gizmo_moved.connect(properties.refresh_values)

	camera_pov.toggled.connect(_on_camera_pov_toggled)
	sub_viewport_container.overlay = camera_pov.get_parent()

	pick_triangle.toggled.connect(_on_pick_triangle_toggled)
	sub_viewport_container.triangle_picked.connect(_on_triangle_picked)

	pick_pixel.toggled.connect(_on_pick_pixel_toggled)
	sub_viewport_container.pixel_picked.connect(_on_pixel_picked)

	save_dialog = FileDialog.new()
	save_dialog.file_mode = FileDialog.FILE_MODE_SAVE_FILE
	save_dialog.access = FileDialog.ACCESS_FILESYSTEM
	save_dialog.title = "Save scene"
	save_dialog.use_native_dialog = true
	save_dialog.add_filter("*.tcc2.glb", "glTF binary")
	save_dialog.file_selected.connect(_save_scene)
	add_child(save_dialog)

	open_dialog = FileDialog.new()
	open_dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	open_dialog.access = FileDialog.ACCESS_FILESYSTEM
	open_dialog.title = "Open scene"
	open_dialog.use_native_dialog = true
	open_dialog.add_filter("*.tcc2.glb", "glTF binary")
	open_dialog.file_selected.connect(_open_scene)
	add_child(open_dialog)

	var on_save_pressed := func() -> void:
		save_dialog.current_file = root.name + ".tcc2.glb"
		save_dialog.popup_centered_ratio(0.6)

	save_button.pressed.connect(on_save_pressed)
	open_button.pressed.connect(open_dialog.popup_centered_ratio.bind(0.6))

func rebuild_hierarchy() -> void:
	hierarchy.clear()
	_items.clear()
	add_item(root, null)

	if sub_viewport_container.render_camera() == null:
		camera_pov.button_pressed = false
	elif camera_pov.button_pressed:
		sub_viewport_container.set_pov(true)

	_refresh_modes()

func add_item(node: Tcc2Node, parent_item: TreeItem) -> void:
	var item := hierarchy.create_item(parent_item)
	item.set_text(0, node.name)
	item.set_metadata(0, node)
	_items[node] = item

	for entry in ICONS:
		if is_instance_of(node, entry[0]):
			item.set_icon(0, entry[1])
			item.set_icon_modulate(0, entry[2])
			break

	for child in node.get_children():
		if child is Tcc2Node:
			add_item(child, item)

func _on_sub_viewport_container_tcc2_node_selected(node: Tcc2Node) -> void:
	if node == null:
		node = root

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
	properties.show_node(node)

func _on_selected() -> void:
	_select(hierarchy.get_selected().get_metadata(0))

func _on_camera_pov_toggled(pressed: bool) -> void:
	_set_pov(pressed)
	_refresh_modes()

func _set_pov(value: bool) -> void:
	camera_pov.set_pressed_no_signal(value)
	sub_viewport_container.set_pov(value)

	if value:
		_items[root].select(0)
		_select(root)

func _on_pick_triangle_toggled(pressed: bool) -> void:
	sub_viewport_container.set_pick_triangle(pressed)

	if pressed:
		_items[root].select(0)
		_select(root)

	_refresh_modes()

func _on_pick_pixel_toggled(pressed: bool) -> void:
	if pressed:
		_set_pov(true)
		sub_viewport_container.set_pick_pixel(true)
	else:
		sub_viewport_container.set_pick_pixel(false)
		_set_pov(false)

	_refresh_modes()

func _refresh_modes() -> void:
	var in_pick_triangle := pick_triangle.button_pressed
	var in_camera_pov := camera_pov.button_pressed and not pick_pixel.button_pressed
	var in_pick_pixel := pick_pixel.button_pressed

	if sub_viewport_container.render_camera() == null:
		camera_pov.disabled = true
		pick_triangle.disabled = true
		pick_pixel.disabled = true
	else:
		camera_pov.disabled = in_pick_triangle or in_pick_pixel
		pick_triangle.disabled = in_camera_pov or in_pick_pixel
		pick_pixel.disabled = in_pick_triangle or in_camera_pov

	for node in _items:
		_items[node].set_selectable(0, not (in_pick_triangle or in_camera_pov or in_pick_pixel))

func _on_triangle_picked(node: Tcc2Geometry, instance: int, face: int) -> void:
	pick_triangle.button_pressed = false
	print("%s: instance %d, face %d" % [node.name, instance, face])

func _on_pixel_picked(x: int, y: int) -> void:
	pick_pixel.button_pressed = false
	print("pixel %d, %d" % [x, y])

func _unhandled_key_input(event: InputEvent) -> void:
	var key := event as InputEventKey

	if key == null or not key.pressed or key.echo:
		return

	if key.keycode == KEY_DELETE and _selected != null and _selected != root:
		_remove(_selected)

func _save_scene(path: String) -> void:
	if path.get_extension().to_lower() != "glb":
		path += ".glb"

	var warnings: Array[String] = []
	var bytes := Tcc2Export.glb(root, warnings)

	var file := FileAccess.open(path, FileAccess.WRITE)

	if file == null:
		push_error("could not write %s: %s"
			% [path, error_string(FileAccess.get_open_error())])
		return

	file.store_buffer(bytes)
	file.close()

	for warning in warnings:
		print("  ", warning)

	root.name = path.get_file().get_basename().trim_suffix(".tcc2")

	if _items.has(root):
		_items[root].set_text(0, root.name)

func _open_scene(path: String) -> void:
	var opened := Tcc2Import.glb(path, true)

	if opened == null:
		return

	_select(null)
	scene_view.remove_child(root)
	root.queue_free()

	root = Tcc2Scene.new()
	
	root.name = path.get_file().get_basename().trim_suffix(".tcc2")

	if _items.has(root):
		_items[root].set_text(0, root.name)
	
	scene_view.add_child(root)

	for child in opened.get_children():
		opened.remove_child(child)
		root.add_child(child)
		_own(child)

	opened.free()
	Tcc2Import.parameters(path, root)

	rebuild_hierarchy()
	
	_items[root].select(0)
	_select(root)


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
	ADD_GROUP,
}

func _on_mouse_select(_pos: Vector2, button: int) -> void:
	if button == MOUSE_BUTTON_RIGHT:
		popup_menu.clear()
		
		popup_menu.add_item("Add object")
		popup_menu.set_item_disabled(0, true)
		popup_menu.add_separator()

		popup_menu.add_item("Sphere", ADD_SPHERE)
		popup_menu.add_item("Cube", ADD_CUBE)
		popup_menu.add_item("Mesh", ADD_MESH)

		popup_menu.add_separator()
		popup_menu.add_item("Point light", ADD_POINT_LIGHT)
		popup_menu.add_item("Directional light", ADD_DIRECTIONAL_LIGHT)
		popup_menu.add_item("Spot light", ADD_SPOT_LIGHT)
		popup_menu.add_item("Ambient light", ADD_AMBIENT_LIGHT)

		popup_menu.add_separator()
		popup_menu.add_item("Perspective camera", ADD_PERSPECTIVE_CAMERA)
		popup_menu.add_item("Orthographic camera", ADD_ORTHOGRAPHIC_CAMERA)

		popup_menu.add_separator()
		popup_menu.add_item("Group", ADD_GROUP)

		popup_menu.reset_size()
		popup_menu.popup_on_parent(Rect2i(get_global_mouse_position(), Vector2i(1, 1)))

func _on_popup_menu_id_pressed(id: int) -> void:
	var target = hierarchy.get_selected().get_metadata(0)

	match id:
		ADD_SPHERE:
			var geometry := Tcc2Geometry.new()
			geometry.kind = Tcc2Geometry.Kind.SPHERE
			_add(target, geometry, "Sphere")

		ADD_CUBE:
			var geometry := Tcc2Geometry.new()
			geometry.kind = Tcc2Geometry.Kind.CUBE
			_add(target, geometry, "Cube")

		ADD_MESH:
			var geometry := Tcc2Geometry.new()
			geometry.kind = Tcc2Geometry.Kind.MESH
			_add(target, geometry, "Mesh")

		ADD_POINT_LIGHT:
			var light := Tcc2Light.new()
			light.kind = Tcc2Light.Kind.POINT
			_add(target, light, "PointLight")

		ADD_DIRECTIONAL_LIGHT:
			var light := Tcc2Light.new()
			light.kind = Tcc2Light.Kind.DIRECTIONAL
			_add(target, light, "DirectionalLight")

		ADD_SPOT_LIGHT:
			var light := Tcc2Light.new()
			light.kind = Tcc2Light.Kind.SPOT
			_add(target, light, "SpotLight")

		ADD_AMBIENT_LIGHT:
			var light := Tcc2Light.new()
			light.kind = Tcc2Light.Kind.AMBIENT
			_add(target, light, "AmbientLight")

		ADD_PERSPECTIVE_CAMERA:
			var cam := Tcc2Camera.new()
			cam.kind = Tcc2Camera.Kind.PERSPECTIVE
			_add(target, cam, "PerspectiveCamera")

		ADD_ORTHOGRAPHIC_CAMERA:
			var cam := Tcc2Camera.new()
			cam.kind = Tcc2Camera.Kind.ORTHOGRAPHIC
			_add(target, cam, "OrthographicCamera")

		ADD_GROUP:
			_add(target, Tcc2Node.new(), "Group")

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

		if Tcc2Import.parameters(file, root):
			properties.refresh_values()

	rebuild_hierarchy()
