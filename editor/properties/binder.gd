class_name Tcc2Binder extends RefCounted

const ELLIPSIS := "…"

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
		button.clip_text = true

		var dialog: FileDialog = null

		for child in button.get_children():
			if child is FileDialog:
				dialog = child

		if dialog == null:
			push_error("%s cannot carry %s" % [control.name, property])
			return

		var hold := func(path: String) -> void:
			var file := path.get_file() if not path.is_empty() else "(none)"

			button.tooltip_text = path
			button.set_meta(&"file", file)
			button.set_meta(&"wish", button.get_theme_stylebox(&"normal")
				.get_minimum_size().x + _advance(button, file))

			var rows := button.get_parent()

			if rows is Container:
				rows.queue_sort()

			_fit(button)

		hold.call(object.get(property))

		button.resized.connect(func(): _fit(button))

		button.pressed.connect(func():
			# Reopening where it left off, which the old dialog could not do: it was
			# built on the press and thrown away after it.
			var current: String = object.get(property)

			if not current.is_empty():
				dialog.current_path = current

			dialog.popup_centered_ratio(0.6))

		dialog.file_selected.connect(func(path: String):
			object.set(property, path)
			hold.call(path))

		_sync.append(func(): hold.call(object.get(property)))
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

	edit.alignment = HORIZONTAL_ALIGNMENT_LEFT

	edit.draw.connect(func() -> void:
		_want(field, edit)
		_rewind(edit)
		_caret(edit))

	edit.focus_exited.connect(func():
		if edit.text.strip_edges().is_empty():
			field.set_deferred(&"value", 0.0))

func _want(field: SpinBox, edit: LineEdit) -> void:
	var wish := ceilf(edit.get_theme_stylebox(&"normal").get_minimum_size().x
		+ _advance(edit, edit.text))

	if field.get_meta(&"wish", -1.0) == wish:
		return

	field.set_meta(&"wish", wish)

	var rows := field.get_parent()

	if rows is Container:
		rows.queue_sort()

func _rewind(edit: LineEdit) -> void:
	if edit.get_scroll_offset() == 0.0:
		return

	var whole := (edit.has_selection() and edit.get_selection_from_column() == 0
		and edit.get_selection_to_column() == edit.text.length())

	if edit.has_focus() and edit.is_editing() and not whole:
		return

	edit.caret_column = 0

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

	var at := (box.content_margin_left + edit.get_scroll_offset()
		+ _advance(edit, edit.text.left(edit.caret_column)))

	var height := font.get_height(size)
	var top := box.content_margin_top

	edit.draw_rect(Rect2(at, top, _advance(edit, under) if under != ""
		else Tcc2Theme.CARET, height), Tcc2Theme.FG)

	if under != "":
		edit.draw_string(font, Vector2(at, top + font.get_ascent(size)), under,
			HORIZONTAL_ALIGNMENT_LEFT, -1, size, Tcc2Theme.INK)

## The width a string takes in a field's own face.
func _advance(control: Control, text: String) -> float:
	return control.get_theme_font(&"font").get_string_size(text,
		HORIZONTAL_ALIGNMENT_LEFT, -1, control.get_theme_font_size(&"font_size")).x

func _fit(button: Button) -> void:
	var file: String = button.get_meta(&"file", "")
	var room := button.size.x - button.get_theme_stylebox(&"normal").get_minimum_size().x
	var text := file

	if room > 0.0 and _advance(button, file) > room:
		var kept := file

		while not kept.is_empty() and _advance(button, ELLIPSIS + kept) > room:
			kept = kept.substr(1)

		text = ELLIPSIS + kept

	if button.text != text:
		button.text = text

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
