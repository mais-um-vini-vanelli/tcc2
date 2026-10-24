class_name Hierarchy extends PanelContainer

static var ICONS: Array = [
	[SceneRoot, preload("res://icons/KindScene.svg"), Color("#d5dee8")],
	[SceneLight, preload("res://icons/KindLight.svg"), Color("#e6c94d")],
	[SceneCamera, preload("res://icons/KindCamera.svg"), Color("#d86481")],
	[SceneGeometry, preload("res://icons/KindGeometry.svg"), Color("#51cc8d")],
	[SceneObject, preload("res://icons/KindScene.svg"), Color("#d5dee8")],
]

signal selected(node: SceneObject)
signal add_pressed(button: Button)
signal changed

@onready var _tree: Tree = %Tree
@onready var _add: Button = %Add
@onready var _delete: Button = %Delete

var _items := {}
var _editable := true
var _root: SceneRoot
var _selected: SceneObject
var _hovered: SceneObject

func _ready() -> void:
	_tree.hide_folding = true
	_tree.gui_input.connect(_on_tree_input)
	_tree.mouse_exited.connect(func(): _hover(null))
	_tree.item_collapsed.connect(func(item: TreeItem): _mark(item.get_metadata(0)))
	_tree.item_selected.connect(_on_item_selected)

	_add.pressed.connect(func(): add_pressed.emit(_add))
	_delete.pressed.connect(func(): _remove(_selected))

	_sync_delete()

func rebuild(root: SceneRoot) -> void:
	_root = root
	_tree.clear()
	_items.clear()
	_hovered = null

	_add_item(root, null)

func select(node: SceneObject) -> void:
	if node == null:
		_tree.deselect_all()
		_set_selected(null)
		return

	var item: TreeItem = _items.get(node)

	if item == null:
		return

	item.select(0)
	_tree.scroll_to_item(item)

func add_node(node: SceneObject, node_name: String) -> void:
	var parent: SceneObject = _selected if _selected != null else _root

	node.name = node_name
	parent.add_child(node, true)

	rebuild(_root)
	select(node)
	changed.emit()

func set_editable(editable: bool) -> void:
	_editable = editable
	_add.disabled = not editable
	_sync_delete()

	for node in _items:
		_items[node].set_selectable(0, editable)

func rename(node: SceneObject) -> void:
	var item: TreeItem = _items.get(node)

	if item != null:
		item.set_text(0, node.name)

func _add_item(node: SceneObject, parent_item: TreeItem) -> void:
	var item := _tree.create_item(parent_item)
	item.set_text(0, node.name)
	item.set_metadata(0, node)
	_items[node] = item

	for child in node.get_children():
		if child is SceneObject:
			_add_item(child, item)

	_mark(node)

func _set_selected(node: SceneObject) -> void:
	var was := _selected
	_selected = node

	if was != null and is_instance_valid(was):
		was.set_selected(false)
		_mark(was)

	if node != null:
		node.set_selected(true)
		_mark(node)

	_sync_delete()
	selected.emit(node)

func _on_item_selected() -> void:
	_set_selected(_tree.get_selected().get_metadata(0))

func _sync_delete() -> void:
	_delete.disabled = (not _editable or _selected == null
		or _selected == _root)

func _mark(node: SceneObject) -> void:
	var item: TreeItem = _items.get(node)

	if item == null:
		return

	var fold := KindGlyph.Fold.LEAF

	if item.get_child_count() > 0:
		fold = KindGlyph.Fold.SHUT if item.collapsed else KindGlyph.Fold.OPEN

	for entry in ICONS:
		if is_instance_of(node, entry[0]):
			item.set_icon(0, KindGlyph.of(entry[1], entry[2], fold,
				item.is_selected(0) or node == _hovered))
			return

func _hover(item: TreeItem) -> void:
	_tree.mouse_default_cursor_shape = (Control.CURSOR_POINTING_HAND
		if item != null and item.is_selectable(0) else Control.CURSOR_ARROW)

	var node: SceneObject = item.get_metadata(0) if item != null else null

	var was := _hovered
	_hovered = node

	if was != null and is_instance_valid(was):
		_mark(was)

	if node != null:
		_mark(node)

func _on_tree_input(event: InputEvent) -> void:
	if event is InputEventMouseMotion:
		_hover(_tree.get_item_at_position((event as InputEventMouseMotion).position))
	elif event is InputEventMouseButton:
		var click := event as InputEventMouseButton

		if click.pressed and click.button_index == MOUSE_BUTTON_LEFT:
			var item := _tree.get_item_at_position(click.position)

			if item != null and item.get_child_count() > 0:
				item.collapsed = not item.collapsed
				_mark(item.get_metadata(0))

func _unhandled_key_input(event: InputEvent) -> void:
	var key := event as InputEventKey

	if key == null or not key.pressed or key.echo:
		return

	if key.keycode == KEY_DELETE and _selected != null and _selected != _root:
		_remove(_selected)

func _remove(node: SceneObject) -> void:
	select(null)

	node.get_parent().remove_child(node)
	node.queue_free()

	rebuild(_root)
	changed.emit()
