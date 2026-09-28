class_name FloatingPanel extends PanelContainer

const RADIUS := 6

const PANEL_BG := Color("#252526")
const TITLE_BG := Color("#1e1e1e")
const HIGHLIGHT := Color("#444444")

@onready var _title: Label = $VBoxContainer/Label
@onready var _body: MarginContainer = $VBoxContainer/MarginContainer

func _ready() -> void:
	add_to_group("ui_panel")

	var bg := _flat(PANEL_BG, RADIUS)
	bg.shadow_color = Color(0, 0, 0, 0.22)
	bg.shadow_size = 8
	# Sem content margin: a barra de titulo encosta na borda do painel.
	add_theme_stylebox_override("panel", bg)

	$VBoxContainer.add_theme_constant_override("separation", 0)

	var bar := _flat(TITLE_BG, 0)
	bar.corner_radius_top_left = RADIUS
	bar.corner_radius_top_right = RADIUS
	bar.content_margin_left = 12
	bar.content_margin_right = 12
	bar.content_margin_top = 7
	bar.content_margin_bottom = 7
	_title.add_theme_stylebox_override("normal", bar)
	_title.add_theme_color_override("font_color", Color(0.86, 0.86, 0.90))

	for m in ["margin_left", "margin_right", "margin_top", "margin_bottom"]:
		_body.add_theme_constant_override(m, 8)

	for c in _body.get_children():
		if c is Tree or c is ScrollContainer:
			c.add_theme_stylebox_override("panel", StyleBoxEmpty.new())

		if c is Tree:
			c.add_theme_constant_override("draw_guides", 0)
			
			var hl := _flat(HIGHLIGHT, 3)
			for state in ["selected", "selected_focus", "hovered", "hovered_dimmed"]:
				c.add_theme_stylebox_override(state, hl)

func _flat(color: Color, radius: int) -> StyleBoxFlat:
	var s := StyleBoxFlat.new()
	s.bg_color = color
	s.set_corner_radius_all(radius)
	return s
