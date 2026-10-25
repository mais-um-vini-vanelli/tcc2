@tool
class_name Form extends VBoxContainer

var node: SceneObject

var _sync: Array[Callable] = []

func _ready() -> void:
	if Engine.is_editor_hint():
		theme = Palette.preview()

func build(for_node: SceneObject) -> void:
	node = for_node

	_sync.clear()

	_build()

func _build() -> void:
	pass

func refresh_values() -> void:
	for sync in _sync:
		sync.call()

func bind(control: Node, object: Object, property: NodePath) -> void:
	if not _holds(object, property):
		return

	if control is PropsSpinBox:
		var field := control as PropsSpinBox

		field.value = object.get_indexed(property)
		field.value_changed.connect(func(v: float):
			object.set_indexed(property, int(v) if field.rounded else v))
		_sync.append(func():
			if not field.editing():
				field.value = object.get_indexed(property))

	elif control is PropsCheckBox:
		var box := control as PropsCheckBox
		box.value = object.get_indexed(property)
		box.toggled.connect(func(on: bool): object.set_indexed(property, on))
		_sync.append(func(): box.value = object.get_indexed(property))

	elif control is PropsOptionButton:
		var menu := control as PropsOptionButton
		menu.selected = object.get_indexed(property)
		menu.item_selected.connect(func(i: int): object.set_indexed(property, i))
		_sync.append(func(): menu.selected = object.get_indexed(property))

	elif control is PropsColorPickerButton:
		var color_button := control as PropsColorPickerButton
		color_button.color = object.get_indexed(property)
		color_button.color_changed.connect(func(c: Color): object.set_indexed(property, c))
		_sync.append(func(): color_button.color = object.get_indexed(property))

	else:
		push_error("%s cannot carry %s" % [control.name, property])

func asset_picker(source: Row, index: Row, object: Object, property: NodePath,
		kind: SceneAssets.Kind) -> Callable:
	var source_button := source.value() as PropsButton
	var index_button := index.value() as PropsOptionButton

	if source_button == null or index_button == null:
		push_error("%s is not an asset picker" % source.name)
		return func(): pass

	var show_ := func():
		var scene: SceneRoot = node.find_scene() if node != null else null
		var assets := scene.assets if scene != null else null

		if assets == null:
			return

		var index_: int = object.get_indexed(property)
		var siblings := assets.siblings(kind, index_)

		source_button.label = assets.file(kind, index_)
		index_button.clear()

		for i in siblings.size():
			index_button.add_item(assets.entry(kind, siblings[i]))
			index_button.set_item_id(i, siblings[i])

			if siblings[i] == index_:
				index_button.selected = i

		index.visible = source.visible and siblings.size() > 1

	index_button.item_selected.connect(func(at: int):
		object.set_indexed(property, index_button.get_item_id(at)))

	source_button.path_selected.connect(func(path: String):
		var scene: SceneRoot = node.find_scene() if node != null else null
		var assets := scene.assets if scene != null else null

		if assets == null:
			return

		var index_: int = Import.images(path, assets) \
			if kind == SceneAssets.Kind.IMAGE else Import.meshes(path, assets)

		if index_ >= 0:
			object.set_indexed(property, index_)
			show_.call())

	show_.call()
	_sync.append(show_)

	return show_

func _holds(object: Object, property: NodePath) -> bool:
	if object.get_indexed(property) != null:
		return true

	push_error("%s names no property of %s" % [property, object])
	return false

func _exit_tree() -> void:
	_sync.clear()
