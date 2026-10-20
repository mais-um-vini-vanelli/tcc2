class_name Palette

const BG := Color("0f1317")
const FG := Color("d5dee8")
const MID := Color("8d9ba7")
const DIM := Color("4a5761")
const INK := Color("0b0e11")
const LIT := Color("ffffff")

const MARKER_WIDTH := 26
const KIND_WIDTH := 14

const FOLD := preload("res://icons/Fold.svg")
const UNFOLD := preload("res://icons/Unfold.svg")
const DROPDOWN := preload("res://icons/Dropdown.svg")

const INDENT := 8

const PAD := 8
const GAP := 4

const CARET := 12

static func build() -> Theme:
	var theme := Theme.new()

	_regions(theme)
	_titles(theme)
	_tree(theme)
	_sections(theme)
	_fields(theme)
	_rows(theme)
	_checkboxes(theme)
	_menus(theme)
	_buttons(theme)
	_scrollbars(theme)
	_splits(theme)
	_spacing(theme)

	return theme

static func _regions(theme: Theme) -> void:
	for type in ["Panel", "PanelContainer"]:
		theme.set_stylebox("panel", type, _box(BG, DIM))

static func _titles(theme: Theme) -> void:
	theme.set_type_variation("PanelTitle", "Label")
	theme.set_color("font_color", "PanelTitle", DIM)

	var box := _box(Color.TRANSPARENT)
	box.border_width_bottom = 1
	box.border_color = DIM
	_margins(box, PAD, GAP)

	theme.set_stylebox("normal", "PanelTitle", box)

static func _tree(theme: Theme) -> void:
	theme.set_stylebox("panel", "Tree", StyleBoxEmpty.new())

	var lit := _box(FG)

	for name in ["selected", "selected_focus", "hovered_selected",
			"hovered_selected_focus", "hovered", "hovered_dimmed"]:
		theme.set_stylebox(name, "Tree", lit)

	for name in ["cursor", "cursor_unfocused"]:
		theme.set_stylebox(name, "Tree", StyleBoxEmpty.new())

	theme.set_color("font_color", "Tree", FG)

	for name in ["font_selected_color", "font_hovered_selected_color",
			"font_hovered_color", "font_hovered_dimmed_color"]:
		theme.set_color(name, "Tree", INK)

	theme.set_color("font_disabled_color", "Tree", DIM)

	theme.set_constant("draw_relationship_lines", "Tree", 0)
	theme.set_constant("draw_guides", "Tree", 0)
	theme.set_constant("icon_max_width", "Tree", KindGlyph.WIDTH)
	theme.set_constant("h_separation", "Tree", GAP)
	theme.set_constant("v_separation", "Tree", GAP)
	theme.set_constant("item_margin", "Tree", INDENT)
	theme.set_constant("inner_item_margin_left", "Tree", PAD)
	theme.set_constant("inner_item_margin_right", "Tree", PAD)

static func _sections(theme: Theme) -> void:
	theme.set_type_variation("Section", "PanelContainer")
	theme.set_type_variation("Subsection", "PanelContainer")
	theme.set_type_variation("SectionHeader", "Button")

	var section := _box(Color.TRANSPARENT)
	_margins(section, 0, GAP)

	theme.set_stylebox("panel", "Section", section)

	var sub := _box(Color.TRANSPARENT)
	_margins(sub, PAD, 0)
	sub.content_margin_right = 0

	theme.set_stylebox("panel", "Subsection", sub)

	var header := StyleBoxEmpty.new()
	_margins(header, PAD, GAP)

	for name in ["normal", "hover", "pressed", "focus", "disabled"]:
		theme.set_stylebox(name, "SectionHeader", header)

	theme.set_color("font_color", "SectionHeader", DIM)
	theme.set_color("font_pressed_color", "SectionHeader", DIM)
	theme.set_color("font_hover_color", "SectionHeader", FG)
	theme.set_color("icon_normal_color", "SectionHeader", DIM)
	theme.set_color("icon_pressed_color", "SectionHeader", DIM)
	theme.set_color("icon_hover_color", "SectionHeader", FG)
	theme.set_constant("h_separation", "SectionHeader", GAP + 2)
	theme.set_constant("icon_max_width", "SectionHeader", MARKER_WIDTH)

	theme.set_icon("fold", "SectionHeader", FOLD)
	theme.set_icon("unfold", "SectionHeader", UNFOLD)

static func _fields(theme: Theme) -> void:
	for type in ["LineEdit", "OptionButton", "FieldButton", "ColorPickerButton"]:
		if type == "FieldButton":
			theme.set_type_variation("FieldButton", "Button")

		for name in ["normal", "hover", "pressed", "hover_pressed", "focus",
				"read_only", "disabled"]:
			theme.set_stylebox(name, type, _field())

		theme.set_color("font_color", type, FG)
		theme.set_color("font_uneditable_color", type, DIM)
		theme.set_color("font_disabled_color", type, DIM)

		for name in ["font_hover_color", "font_pressed_color",
				"font_focus_color", "font_hover_pressed_color"]:
			theme.set_color(name, type, LIT)

	for name in ["normal", "hover", "pressed", "focus", "disabled"]:
		theme.set_stylebox(name, "ColorPickerButton",
			_swatch(FG if name in ["pressed", "focus"] else DIM))

	theme.set_stylebox("focus", "LineEdit", _field())
	theme.set_color("caret_color", "LineEdit", FG)

	theme.set_constant("caret_width", "LineEdit", 2)
	theme.set_color("font_placeholder_color", "LineEdit", DIM)

	theme.set_color("selection_color", "LineEdit", FG)
	theme.set_color("font_selected_color", "LineEdit", INK)

	theme.set_constant("minimum_character_width", "LineEdit", 4)

	for name in ["normal", "read_only", "focus"]:
		var caret_room: StyleBox = theme.get_stylebox(name, "LineEdit").duplicate()
		caret_room.content_margin_right = CARET
		theme.set_stylebox(name, "LineEdit", caret_room)

	theme.set_icon("arrow", "OptionButton", DROPDOWN)
	theme.set_color("font_icon_color", "OptionButton", MID)
	theme.set_color("icon_normal_color", "OptionButton", MID)

	for name in ["icon_hover_color", "icon_pressed_color", "icon_focus_color"]:
		theme.set_color(name, "OptionButton", LIT)
	theme.set_constant("arrow_margin", "OptionButton", GAP)
	theme.set_constant("modulate_arrow", "OptionButton", 1)

	var blank := StyleBoxEmpty.new()

	for name in ["up_background", "up_background_hovered", "up_background_pressed",
			"up_background_disabled", "down_background", "down_background_hovered",
			"down_background_pressed", "down_background_disabled",
			"field_and_buttons_separator", "up_down_buttons_separator"]:
		theme.set_stylebox(name, "SpinBox", blank)

	for name in ["up_icon_modulate", "up_hover_icon_modulate", "up_pressed_icon_modulate",
			"up_disabled_icon_modulate", "down_icon_modulate", "down_hover_icon_modulate",
			"down_pressed_icon_modulate", "down_disabled_icon_modulate"]:
		theme.set_color(name, "SpinBox", Color.TRANSPARENT)

	theme.set_constant("buttons_width", "SpinBox", 0)
	theme.set_constant("field_and_buttons_separation", "SpinBox", 0)
	theme.set_constant("set_min_buttons_width_from_icons", "SpinBox", 0)

static func _rows(theme: Theme) -> void:
	theme.set_type_variation("Rows", "Container")
	theme.set_color("leader", "Rows", DIM)
	theme.set_constant("h_separation", "Rows", PAD)
	theme.set_constant("v_separation", "Rows", GAP)

static func _checkboxes(theme: Theme) -> void:
	var rest := _box(Color.TRANSPARENT)
	_margins(rest, GAP / 2, GAP / 2)
	rest.content_margin_right = CARET

	for name in ["normal", "pressed", "hover", "hover_pressed", "focus",
			"disabled"]:
		theme.set_stylebox(name, "CheckBox", rest)

	var nothing := PlaceholderTexture2D.new()
	nothing.size = Vector2.ZERO

	for name in ["checked", "unchecked", "checked_disabled", "unchecked_disabled",
			"radio_checked", "radio_unchecked", "radio_checked_disabled",
			"radio_unchecked_disabled"]:
		theme.set_icon(name, "CheckBox", nothing)

	theme.set_color("font_color", "CheckBox", FG)
	theme.set_color("font_pressed_color", "CheckBox", FG)
	theme.set_color("font_disabled_color", "CheckBox", DIM)

	for name in ["font_hover_color", "font_hover_pressed_color",
			"font_focus_color"]:
		theme.set_color(name, "CheckBox", LIT)
	theme.set_constant("h_separation", "CheckBox", 0)

static func _menus(theme: Theme) -> void:
	for type in ["PopupMenu", "PopupPanel"]:
		theme.set_stylebox("panel", type, _box(BG, DIM))

	theme.set_stylebox("hover", "PopupMenu", _box(FG))
	theme.set_color("font_color", "PopupMenu", FG)
	theme.set_color("font_hover_color", "PopupMenu", INK)
	theme.set_color("font_separator_color", "PopupMenu", DIM)
	theme.set_color("font_disabled_color", "PopupMenu", DIM)

	var nothing := PlaceholderTexture2D.new()
	nothing.size = Vector2.ZERO

	for name in ["checked", "unchecked", "checked_disabled", "unchecked_disabled",
			"radio_checked", "radio_unchecked", "radio_checked_disabled",
			"radio_unchecked_disabled"]:
		theme.set_icon(name, "PopupMenu", nothing)

	theme.set_constant("icon_max_width", "PopupMenu", MARKER_WIDTH)
	theme.set_constant("h_separation", "PopupMenu", GAP)
	theme.set_constant("item_start_padding", "PopupMenu", GAP)
	theme.set_constant("item_end_padding", "PopupMenu", GAP * 2)

	var tip := _box(BG, DIM)
	_margins(tip, PAD, GAP)

	theme.set_stylebox("panel", "TooltipPanel", tip)

static func _buttons(theme: Theme) -> void:
	theme.set_constant("icon_max_width", "Button", 32)

	theme.set_type_variation("OverlayButton", "Button")

	var rest := _box(BG, DIM)
	_margins(rest, GAP, GAP)

	var lit := _box(FG, FG)
	_margins(lit, GAP, GAP)

	var brightest := _box(LIT, LIT)
	_margins(brightest, GAP, GAP)

	theme.set_stylebox("normal", "OverlayButton", rest)
	theme.set_stylebox("disabled", "OverlayButton", rest)
	theme.set_stylebox("focus", "OverlayButton", _box(Color.TRANSPARENT))
	theme.set_stylebox("hover", "OverlayButton", lit)
	theme.set_stylebox("pressed", "OverlayButton", lit)
	theme.set_stylebox("hover_pressed", "OverlayButton", brightest)

	theme.set_color("icon_normal_color", "OverlayButton", FG)
	theme.set_color("icon_disabled_color", "OverlayButton", DIM)

	for name in ["icon_hover_color", "icon_pressed_color",
			"icon_hover_pressed_color", "icon_focus_color"]:
		theme.set_color(name, "OverlayButton", INK)

	theme.set_color("font_color", "OverlayButton", FG)
	theme.set_color("font_disabled_color", "OverlayButton", MID)

	for name in ["font_hover_color", "font_pressed_color",
			"font_hover_pressed_color", "font_focus_color"]:
		theme.set_color(name, "OverlayButton", INK)

	theme.set_font_size("font_size", "OverlayButton", 24)

	theme.set_constant("icon_max_width", "OverlayButton", 32)

static func _splits(theme: Theme) -> void:
	for type in ["SplitContainer", "HSplitContainer", "VSplitContainer"]:
		theme.set_constant("separation", type, 0)
		theme.set_constant("minimum_grab_thickness", type, 16)
		theme.set_constant("autohide", type, 0)
		theme.set_stylebox("split_bar_background", type, StyleBoxEmpty.new())

	var nothing := PlaceholderTexture2D.new()
	nothing.size = Vector2.ZERO

	for name in ["grabber", "touch_dragger", "h_grabber", "v_grabber",
			"h_touch_dragger", "v_touch_dragger"]:
		for type in ["SplitContainer", "HSplitContainer", "VSplitContainer"]:
			theme.set_icon(name, type, nothing)

static func _scrollbars(theme: Theme) -> void:
	theme.set_stylebox("panel", "ScrollContainer", StyleBoxEmpty.new())

	for type in ["VScrollBar", "HScrollBar"]:
		for name in ["scroll", "scroll_focus"]:
			theme.set_stylebox(name, type, _box(Color.TRANSPARENT))

		theme.set_stylebox("grabber", type, _box(DIM))
		theme.set_stylebox("grabber_highlight", type, _box(MID))
		theme.set_stylebox("grabber_pressed", type, _box(FG))

static func _spacing(theme: Theme) -> void:
	theme.set_color("font_color", "Label", MID)

	theme.set_constant("separation", "VBoxContainer", GAP)
	theme.set_constant("separation", "HBoxContainer", PAD)
	theme.set_constant("h_separation", "GridContainer", PAD)
	theme.set_constant("v_separation", "GridContainer", GAP)

	for side in ["left", "top", "right", "bottom"]:
		theme.set_constant("margin_" + side, "MarginContainer", PAD)

static func _box(fill: Color, border := Color.TRANSPARENT) -> StyleBoxFlat:
	var box := StyleBoxFlat.new()
	box.bg_color = fill
	box.anti_aliasing = false

	if border.a > 0.0:
		box.border_color = border
		box.set_border_width_all(1)

	return box

static func _field(fill := Color.TRANSPARENT) -> StyleBoxFlat:
	var box := _box(fill)
	_margins(box, GAP, GAP - 1)
	return box

static func _swatch(border: Color) -> StyleBoxFlat:
	var box := _box(Color.TRANSPARENT, border)
	_margins(box, GAP, GAP - 1)
	return box

static func _margins(box: StyleBox, horizontal: int, vertical: int) -> void:
	box.content_margin_left = horizontal
	box.content_margin_right = horizontal
	box.content_margin_top = vertical
	box.content_margin_bottom = vertical
