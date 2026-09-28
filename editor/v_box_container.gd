extends VBoxContainer

const AXIS_COLORS := [ViewportGrid.X_COLOR, ViewportGrid.Y_COLOR, ViewportGrid.Z_COLOR]

var _shown: Node
var _sync: Array[Callable] = []

func show_node(node: Node) -> void:
	for child in get_children():
		remove_child(child)
		child.queue_free()

	_shown = node
	_sync.clear()

	if node == null:
		return

	_transform_rows(node)
	_object_rows(node)

func _refresh() -> void:
	if is_instance_valid(_shown):
		show_node.call_deferred(_shown)

func refresh_values() -> void:
	for sync in _sync:
		sync.call()

func _transform_rows(node: Node) -> void:
	if node is Tcc2Node and node is not Tcc2Scene:
		_vector_row("position",
			func(): return node.position,
			func(v): node.position = v)

		_vector_row("rotation",
			func(): return node.rotation_degrees,
			func(v): node.rotation_degrees = v)

		_vector_row("scale",
			func(): return node.scale,
			func(v): node.scale = v)

		add_child(HSeparator.new())

func _object_rows(object: Object, prefix := "") -> void:
	for prop in object.get_property_list():
		if not (prop.usage & PROPERTY_USAGE_SCRIPT_VARIABLE) or not (prop.usage & PROPERTY_USAGE_EDITOR):
			continue

		var value = object.get(prop.name)

		if value is Resource:
			if not value.property_list_changed.is_connected(_refresh):
				value.property_list_changed.connect(_refresh, CONNECT_ONE_SHOT)

			add_child(HSeparator.new())

			var label := Label.new()
			label.text = prefix + prop.name
			add_child(label)

			_object_rows(value, prefix + prop.name + " / ")
		else:
			_add_row(object, prop)

func _vector_row(text: String, getter: Callable, setter: Callable) -> void:
	var box := VBoxContainer.new()

	var label := Label.new()
	label.text = text
	box.add_child(label)

	var row := HBoxContainer.new()
	row.add_theme_constant_override("separation", 4)

	for axis in 3:
		var get_axis := func(): return getter.call()[axis]
		var set_axis := func(v: float):
			var current: Vector3 = getter.call()
			current[axis] = v
			setter.call(current)

		var axis_label := Label.new()
		axis_label.text = "xyz"[axis]
		axis_label.add_theme_color_override("font_color", AXIS_COLORS[axis])
		row.add_child(axis_label)

		var field := _float_field(get_axis, set_axis)

		_sync.append(func():
			if not field.has_focus():
				field.text = String.num(get_axis.call(), 3))

		row.add_child(field)

	box.add_child(row)
	add_child(box)

func _float_field(getter: Callable, setter: Callable, min_v := - INF, max_v := INF) -> LineEdit:
	var field := LineEdit.new()
	field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
	field.custom_minimum_size.x = 40
	field.alignment = HORIZONTAL_ALIGNMENT_RIGHT
	field.text = String.num(getter.call(), 3)

	var commit := func():
		if field.text.is_valid_float():
			setter.call(clampf(field.text.to_float(), min_v, max_v))

		field.text = String.num(getter.call(), 3)

	field.text_submitted.connect(func(_t): field.release_focus())
	field.focus_exited.connect(commit)
	return field

func _add_row(object: Object, prop: Dictionary) -> void:
	var row := HBoxContainer.new()

	var label := Label.new()
	label.text = prop.name
	label.custom_minimum_size.x = 120
	row.add_child(label)

	row.add_child(_editor_for(object, prop))

	add_child(row)

func _editor_for(object: Object, prop: Dictionary) -> Control:
	match prop.type:
		TYPE_BOOL:
			var cb := CheckBox.new()
			cb.button_pressed = object.get(prop.name)
			cb.toggled.connect(func(v): object.set(prop.name, v))
			return cb

		TYPE_FLOAT:
			var sb := SpinBox.new()
			sb.step = 0.01
			sb.allow_greater = true
			sb.allow_lesser = true

			if prop.hint == PROPERTY_HINT_RANGE:
				var parts := String(prop.hint_string).split(",")
				sb.min_value = float(parts[0])
				sb.max_value = float(parts[1])
				sb.allow_greater = false
				sb.allow_lesser = false

			sb.value = object.get(prop.name)
			sb.value_changed.connect(func(v): object.set(prop.name, v))
			return sb

		TYPE_INT:
			if prop.hint == PROPERTY_HINT_ENUM:
				var ob := OptionButton.new()
				for _name in String(prop.hint_string).split(","):
					ob.add_item(_name)
				ob.selected = object.get(prop.name)
				ob.item_selected.connect(func(i): object.set(prop.name, i))
				return ob

			var sb := SpinBox.new()
			sb.step = 1
			sb.rounded = true

			if prop.hint == PROPERTY_HINT_RANGE:
				var parts := String(prop.hint_string).split(",")
				sb.min_value = float(parts[0])
				sb.max_value = float(parts[1])
			else:
				sb.allow_greater = true
				sb.allow_lesser = true

			sb.value = object.get(prop.name)
			sb.value_changed.connect(func(v): object.set(prop.name, int(v)))
			return sb

		TYPE_COLOR:
			var cp := ColorPickerButton.new()
			cp.color = object.get(prop.name)
			cp.color_changed.connect(func(c): object.set(prop.name, c))
			return cp

		TYPE_STRING:
			if prop.hint == PROPERTY_HINT_FILE:
				var button := Button.new()
				button.size_flags_horizontal = Control.SIZE_EXPAND_FILL
				button.clip_text = true
				button.text = _file_text(object.get(prop.name))
				button.pressed.connect(func(): _pick_file(object, prop, button))
				return button

			var field := LineEdit.new()
			field.size_flags_horizontal = Control.SIZE_EXPAND_FILL
			field.text = object.get(prop.name)
			field.text_submitted.connect(func(t): object.set(prop.name, t))
			return field

	return Label.new()

func _file_text(path: String) -> String:
	return path.get_file() if not path.is_empty() else "(none)"

func _pick_file(object: Object, prop: Dictionary, button: Button) -> void:
	var dialog := FileDialog.new()
	dialog.file_mode = FileDialog.FILE_MODE_OPEN_FILE
	dialog.access = FileDialog.ACCESS_FILESYSTEM
	dialog.use_native_dialog = true
	dialog.filters = PackedStringArray(String(prop.hint_string).split(","))

	dialog.file_selected.connect(func(path: String):
		object.set(prop.name, path)
		button.text = _file_text(path)
		dialog.queue_free())
	dialog.canceled.connect(dialog.queue_free)

	add_child(dialog)
	dialog.popup_centered_ratio(0.6)
