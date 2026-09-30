class_name Tcc2Binder extends RefCounted

var _sync: Array[Callable] = []
var _bound: Array = []

func bind(control: Node, object: Object, property: StringName) -> void:
	if not _holds(object, property):
		return

	if control is SpinBox:
		var field := control as SpinBox
		field.value = object.get(property)
		field.value_changed.connect(func(v: float):
			object.set(property, int(v) if field.rounded else v))
		_sync.append(func():
			if not field.get_line_edit().has_focus():
				field.value = object.get(property))

	elif control is CheckBox:
		var box := control as CheckBox
		box.set_pressed_no_signal(object.get(property))
		box.toggled.connect(func(on: bool): object.set(property, on))
		_sync.append(func(): box.set_pressed_no_signal(object.get(property)))

	elif control is OptionButton:
		var menu := control as OptionButton
		menu.selected = object.get(property)
		menu.item_selected.connect(func(i: int): object.set(property, i))
		_sync.append(func(): menu.selected = object.get(property))

	elif control is ColorPickerButton:
		var picker := control as ColorPickerButton
		picker.color = object.get(property)
		picker.color_changed.connect(func(c: Color): object.set(property, c))
		_sync.append(func(): picker.color = object.get(property))

	elif control is Button:
		# A plain button picks a file, and shows the name of the one it holds..
		var button := control as Button

		var file_text = func(path: String) -> String:
			return path.get_file() if not path.is_empty() else "(none)"

		var dialog: FileDialog = null

		for child in button.get_children():
			if child is FileDialog:
				dialog = child

		if dialog == null:
			push_error("%s cannot carry %s" % [control.name, property])
			return

		button.text = file_text.call(object.get(property))

		button.pressed.connect(func():
			# Reopening where it left off, which the old dialog could not do: it was
			# built on the press and thrown away after it.
			var current: String = object.get(property)

			if not current.is_empty():
				dialog.current_path = current

			dialog.popup_centered_ratio(0.6))

		dialog.file_selected.connect(func(path: String):
			object.set(property, path)
			button.text = file_text.call(path))

		_sync.append(func(): button.text = file_text.call(object.get(property)))
	else:
		push_error("%s cannot carry %s" % [control.name, property])

# One axis of a Vector3 property. Separate from bind() because `position_x` is not
# a property of anything.
func bind_axis(control: Node, object: Object, property: StringName, axis: int) -> void:
	if not _holds(object, property):
		return

	if not control is SpinBox:
		push_error("%s cannot carry an axis of %s" % [control.name, property])
		return

	var field := control as SpinBox
	field.value = object.get(property)[axis]

	field.value_changed.connect(func(v: float):
		var vector: Vector3 = object.get(property)
		vector[axis] = v
		object.set(property, vector))

	_sync.append(func():
		if not field.get_line_edit().has_focus():
			field.value = object.get(property)[axis])

func _holds(object: Object, property: StringName) -> bool:
	if String(property) in object:
		return true

	push_error("%s names no property of %s" % [property, object])
	return false

func follow(source: Signal, action: Callable) -> void:
	action.call()
	source.connect(action)
	_bound.append([source, action])

func refresh() -> void:
	for sync in _sync:
		sync.call()

func unbind() -> void:
	for binding in _bound:
		binding[0].disconnect(binding[1])

	_bound.clear()
	_sync.clear()