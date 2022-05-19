import bpy

# canonically __name__ or __package__ is used but 
# there was an issue I didn't have time to investigate
_preferences_id = "blender"

def get_preferences(context):
    return context.preferences.addons[_preferences_id].preferences

class VIEW3D_PT_preferences(bpy.types.Panel):
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = "Legion"
    bl_label = "Preferences"

    def draw(self, context):
        layout = self.layout
        lgn_prefs = get_preferences(context)
        layout.prop(lgn_prefs, "server_address")

class LgnPreferences(bpy.types.AddonPreferences):
    bl_idname = _preferences_id
    server_address: bpy.props.StringProperty(
        name="Server Address",
        default="[::1]:50051"
    )

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "server_address")

def register():
    bpy.utils.register_class(VIEW3D_PT_preferences)
    bpy.utils.register_class(LgnPreferences)

def unregister():
    bpy.utils.unregister_class(VIEW3D_PT_preferences)
    bpy.utils.unregister_class(LgnPreferences)