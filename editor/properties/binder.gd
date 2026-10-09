class_name Tcc2Binder extends RefCounted

var _sync: Array[Callable] = []
var _bound: Array = []

func bind(control: Node, object: Object, property: StringName) -> void:
	if not _holds(object, property):
		return

	var clickable := control as Control

	if clickable != null and not control is SpinBox:
		clickable.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND

	if control is SpinBox:
		var field := control as SpinBox
		
		field.select_all_on_focus = true
		field.alignment = HORIZONTAL_ALIGNMENT_RIGHT
		_elastic(field)

		field.value = object.get(property)
		field.value_changed.connect(func(v: float):
			object.set(property, int(v) if field.rounded else v))
		_sync.append(func():
			if not field.get_line_edit().has_focus():
				field.value = object.get(property))

	elif control is Tcc2Check:
		var box := control as Tcc2Check
		box.value = object.get(property)
		box.toggled.connect(func(on: bool): object.set(property, on))
		_sync.append(func(): box.value = object.get(property))

	elif control is OptionButton:
		var menu := control as OptionButton
		
		menu.alignment = HORIZONTAL_ALIGNMENT_RIGHT
		menu.fit_to_longest_item = false
		menu.selected = object.get(property)
		menu.item_selected.connect(func(i: int): object.set(property, i))
		_sync.append(func(): menu.selected = object.get(property))

	elif control is ColorPickerButton:
		var picker := control as ColorPickerButton

		var font_size := picker.get_theme_font_size(&"font_size")
		picker.custom_minimum_size.y = picker.get_theme_font(&"font").get_height(font_size) + picker.get_theme_stylebox(&"normal").get_minimum_size().y

		picker.color = object.get(property)
		picker.color_changed.connect(func(c: Color): object.set(property, c))
		_sync.append(func(): picker.color = object.get(property))

	elif control is Button:
		# A plain button picks a file, and shows the name of the one it holds.
		var button := control as Button
		button.theme_type_variation = &"FieldButton"
		button.alignment = HORIZONTAL_ALIGNMENT_RIGHT

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
	
	field.select_all_on_focus = true
	field.alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_elastic(field)

	field.value = object.get(property)[axis]

	field.value_changed.connect(func(v: float):
		var vector: Vector3 = object.get(property)
		vector[axis] = v
		object.set(property, vector))

	_sync.append(func():
		if not field.get_line_edit().has_focus():
			field.value = object.get(property)[axis])

func _elastic(field: SpinBox) -> void:
	var edit := field.get_line_edit()

	edit.caret_blink = false
	edit.add_theme_color_override(&"caret_color", Color.TRANSPARENT)
	edit.add_theme_constant_override(&"minimum_character_width", 0)

	edit.draw.connect(func() -> void:
		field.custom_minimum_size.x = ceilf(edit.get_theme_stylebox(&"normal")
			.get_minimum_size().x + _advance(edit, edit.text))
		_caret(edit))

## The block caret, drawn the way a terminal drew one: the cell under the
## caret fills, and the character standing in it is struck again in the
## panel's own colour.
func _caret(edit: LineEdit) -> void:
	if not edit.has_focus() or not edit.is_editing():
		return

	var font := edit.get_theme_font(&"font")
	var size := edit.get_theme_font_size(&"font_size")
	var box := edit.get_theme_stylebox(&"normal")
	var under := edit.text.substr(edit.caret_column, 1)

	# Get the caret position.
	var at := (edit.size.x - box.content_margin_right - _advance(edit, edit.text)
		+ _advance(edit, edit.text.left(edit.caret_column)))

	var height := font.get_height(size)
	var top := box.content_margin_top

	edit.draw_rect(Rect2(at, top, _advance(edit, under) if under != ""
		else Tcc2Theme.CARET, height), Tcc2Theme.FG)

	if under != "":
		edit.draw_string(font, Vector2(at, top + font.get_ascent(size)), under,
			HORIZONTAL_ALIGNMENT_LEFT, -1, size, Tcc2Theme.INK)

## The width a string takes in a field's own face.
func _advance(edit: LineEdit, text: String) -> float:
	return edit.get_theme_font(&"font").get_string_size(text,
		HORIZONTAL_ALIGNMENT_LEFT, -1, edit.get_theme_font_size(&"font_size")).x

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
