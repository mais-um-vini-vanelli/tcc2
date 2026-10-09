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
@onready var add_button: Button = $Layout/Sidebar/Hierarchy/VBoxContainer/Title/Add

const CURSORS: Array = [
	[preload("res://icons/Arrow.svg"), Input.CURSOR_ARROW, Vector2(6, 4)],
	[preload("res://icons/Hand.svg"), Input.CURSOR_POINTING_HAND, Vector2(8, 1)],
	[preload("res://icons/IBeam.svg"), Input.CURSOR_IBEAM, Vector2(12, 12)],
	[preload("res://icons/Forbidden.svg"), Input.CURSOR_FORBIDDEN, Vector2(12, 12)],
	[preload("res://icons/Pan.svg"), Input.CURSOR_MOVE, Vector2(11, 11)],
	[preload("res://icons/SplitH.svg"), Input.CURSOR_HSPLIT, Vector2(11, 5)],
	[preload("res://icons/SplitV.svg"), Input.CURSOR_VSPLIT, Vector2(5, 11)],
]

static var ICONS: Array = [
	[Tcc2Scene, preload("res://icons/KindScene.svg"), Color("#d5dee8")],
	[Tcc2Light, preload("res://icons/KindLight.svg"), Color("#e6c94d")],
	[Tcc2Camera, preload("res://icons/KindCamera.svg"), Color("#d86481")],
	[Tcc2Geometry, preload("res://icons/KindGeometry.svg"), Color("#51cc8d")],
]

var root: Tcc2Scene
var save_dialog: FileDialog
var open_dialog: FileDialog
var _items := {}
var _selected: Tcc2Node
var _hovered: Tcc2Node

func _ready() -> void:
	for cursor in CURSORS:
		Input.set_custom_mouse_cursor(_rimmed(cursor[0]), cursor[1], cursor[2])
	
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

	hierarchy.hide_folding = true
	hierarchy.gui_input.connect(_on_hierarchy_input)
	hierarchy.mouse_exited.connect(func(): _hover(null))
	hierarchy.item_collapsed.connect(func(item: TreeItem): _mark(item.get_metadata(0)))
	
	rebuild_hierarchy()
	
	hierarchy.item_selected.connect(_on_selected)
	_items[root].select(0)
	_select(root)

	sub_viewport_container.tcc2_node_selected.connect(_on_sub_viewport_container_tcc2_node_selected)
	add_button.pressed.connect(_on_add_pressed)
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

func _rimmed(texture: Texture2D) -> ImageTexture:
	var shape := texture.get_image()
	shape.convert(Image.FORMAT_RGBA8)

	var touches := func(shape: Image, x: int, y: int) -> bool:
		for dy in range(-1, 2):
			for dx in range(-1, 2):
				var at := Vector2i(x + dx, y + dy)

				if at.x < 0 or at.y < 0 or at.x >= shape.get_width() or at.y >= shape.get_height():
					continue

				if shape.get_pixel(at.x, at.y).a > 0.0:
					return true

		return false

	var size := shape.get_size()
	var rim := Image.create_empty(size.x, size.y, false, Image.FORMAT_RGBA8)

	for y in size.y:
		for x in size.x:
			if shape.get_pixel(x, y).a == 0.0 and touches.call(shape, x, y):
				rim.set_pixel(x, y, Tcc2Theme.INK)

	rim.blend_rect(shape, Rect2i(Vector2i.ZERO, size), Vector2i.ZERO)

	return ImageTexture.create_from_image(rim)

func rebuild_hierarchy() -> void:
	hierarchy.clear()
	_items.clear()
	_hovered = null
	add_item(root, null)

	if sub_viewport_container.render_camera() == null:
		camera_pov.button_pressed = false
	elif camera_pov.button_pressed or pick_pixel.button_pressed:
		sub_viewport_container.set_pov(true)

	_refresh_modes()

func add_item(node: Tcc2Node, parent_item: TreeItem) -> void:
	var item := hierarchy.create_item(parent_item)
	item.set_text(0, node.name)
	item.set_metadata(0, node)
	_items[node] = item

	for child in node.get_children():
		if child is Tcc2Node:
			add_item(child, item)

	_mark(node)

func _mark(node: Tcc2Node) -> void:
	var item: TreeItem = _items.get(node)

	if item == null:
		return

	var fold := Tcc2KindGlyph.Fold.LEAF

	if item.get_child_count() > 0:
		fold = Tcc2KindGlyph.Fold.SHUT if item.collapsed else Tcc2KindGlyph.Fold.OPEN

	for entry in ICONS:
		if is_instance_of(node, entry[0]):
			item.set_icon(0, Tcc2KindGlyph.of(entry[1], entry[2], fold,
				item.is_selected(0) or node == _hovered))
			return

func _hover(item: TreeItem) -> void:
	hierarchy.mouse_default_cursor_shape = (Control.CURSOR_POINTING_HAND
		if item != null and item.is_selectable(0) else Control.CURSOR_ARROW)

	var node: Tcc2Node = item.get_metadata(0) if item != null else null

	if node == _hovered:
		return

	var was := _hovered
	_hovered = node

	if was != null and is_instance_valid(was):
		_mark(was)

	if node != null:
		_mark(node)

func _on_hierarchy_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		_hover(hierarchy.get_item_at_position((event as InputEventMouseMotion).position))
	elif event is InputEventMouseButton:
		var click := event as InputEventMouseButton

		if click.pressed and click.button_index == MOUSE_BUTTON_LEFT:
			var item := hierarchy.get_item_at_position(click.position)

			if item != null and item.get_child_count() > 0:
				item.collapsed = not item.collapsed
				_mark(item.get_metadata(0))

func _on_sub_viewport_container_tcc2_node_selected(node: Tcc2Node) -> void:
	if node == null:
		node = root

	var item: TreeItem = _items[node]
	item.select(0)
	_select(node)
	hierarchy.scroll_to_item(item)

func _select(node: Tcc2Node) -> void:
	var was := _selected

	if _selected != null and is_instance_valid(_selected):
		_selected.set_selected(false)

	_selected = node

	if node != null:
		node.set_selected(true)

	if was != null and is_instance_valid(was):
		_mark(was)

	if node != null:
		_mark(node)

	sub_viewport_container.set_target(node)
	properties.show_node(node)

func _on_selected() -> void:
	_select(hierarchy.get_selected().get_metadata(0))

func _on_camera_pov_toggled(pressed: bool) -> void:
	_mode(camera_pov, pressed)
	sub_viewport_container.set_pov(pressed)

func _on_pick_triangle_toggled(pressed: bool) -> void:
	_mode(pick_triangle, pressed)
	sub_viewport_container.set_pick_triangle(pressed)

func _on_pick_pixel_toggled(pressed: bool) -> void:
	_mode(pick_pixel, pressed)
	sub_viewport_container.set_pov(pressed)
	sub_viewport_container.set_pick_pixel(pressed)

func _mode(entering: Button, pressed: bool) -> void:
	if pressed:
		for button in [camera_pov, pick_triangle, pick_pixel]:
			if button != entering and button.button_pressed:
				button.button_pressed = false

		_items[root].select(0)
		_select(root)

	_refresh_modes.call_deferred()

func _refresh_modes() -> void:
	var disable_tree := (camera_pov.button_pressed or pick_triangle.button_pressed
		or pick_pixel.button_pressed)
	var blind := sub_viewport_container.render_camera() == null

	camera_pov.disabled = blind
	pick_triangle.disabled = blind
	pick_pixel.disabled = blind

	for node in _items:
		_items[node].set_selectable(0, not disable_tree)

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

func _on_add_pressed() -> void:
	popup_menu.clear()

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

	var corner := add_button.get_global_rect().end

	popup_menu.popup_on_parent(Rect2i(Vector2i(corner)
		- Vector2i(int(popup_menu.size.x), 0), Vector2i(1, 1)))

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
