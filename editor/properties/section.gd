class_name Tcc2Section extends PanelContainer

var title: String:
	get:
		var header := _header()
		return header.text if header != null else ""
	set(value):
		var header := _header()

		if header != null:
			header.text = value

func _ready() -> void:
	var header := _header()
	var body := _body()

	if header == null or body == null:
		return

	var arrow := func(open: bool) -> Texture2D:
		var icon := &"arrow" if open else &"arrow_collapsed"
		return get_theme_icon(icon, &"Tree")

	body.visible = false
	header.button_pressed = false
	header.icon = arrow.call(false)

	header.toggled.connect(func(on: bool):
		body.visible = on
		header.icon = arrow.call(on))

func _header() -> Button:
	return get_node_or_null("Stack/Header") as Button

func _body() -> Control:
	return get_node_or_null("Stack/Body") as Control

func show_row(control: Control, on: bool) -> void:
	control.visible = on

	var label := control.get_parent().get_child(control.get_index() - 1) as Control

	if label != null:
		label.visible = on
