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
		var picker := control as PropsColorPickerButton
		picker.color = object.get_indexed(property)
		picker.color_changed.connect(func(c: Color): object.set_indexed(property, c))
		_sync.append(func(): picker.color = object.get_indexed(property))

	elif control is PropsButton:
		var button := control as PropsButton
		button.path = object.get_indexed(property)
		button.path_selected.connect(func(path: String):
			object.set_indexed(property, path))
		_sync.append(func(): button.path = object.get_indexed(property))

	else:
		push_error("%s cannot carry %s" % [control.name, property])

func _holds(object: Object, property: NodePath) -> bool:
	if object.get_indexed(property) != null:
		return true

	push_error("%s names no property of %s" % [property, object])
	return false

func _exit_tree() -> void:
	_sync.clear()
