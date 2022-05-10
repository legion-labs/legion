import os
import bpy
from preferences import get_preferences
import sonora.client
import resource_browser_pb2
import resource_browser_pb2_grpc
import source_control_pb2
import source_control_pb2_grpc

class Lgn_UL_ObjectsToImport(bpy.types.UIList):
    def draw_item(self, context, layout, data, item, icon, active_data, active_propname):
        layout.label(text="{} {}".format(item.asset_name, item.id))

class AssetProperties(bpy.types.PropertyGroup):
    id: bpy.props.StringProperty(
        name="id",
        default=""
    )
    asset_name: bpy.props.StringProperty(
        name="Asset Name",
        default="example"
    )

class PullProperties(bpy.types.PropertyGroup):
    assets: bpy.props.CollectionProperty(
        type=AssetProperties,
        override={'LIBRARY_OVERRIDABLE', 'USE_INSERTION'}
    )
    assets_index: bpy.props.IntProperty()

class LgnListAssetsOperator(bpy.types.Operator):
    bl_idname = "lgn.list_assets_operator"
    bl_label = "List LE assets"

    def execute(self, context):
        server_address = get_preferences(context).server_address
        print("connecting to " + server_address)
        context.scene.pull_properties.assets.clear()
        with sonora.client.insecure_web_channel(server_address) as channel:
            rb_stub = resource_browser_pb2_grpc.ResourceBrowserStub(channel)
            response = rb_stub.ListDCCAssets(resource_browser_pb2.ListDCCAssetsRequest(dcc_name="blender"))
            for asset in response.assets:
                print("{}".format(asset))
                prop = context.scene.pull_properties.assets.add()
                prop.id = asset.id
                prop.asset_name = asset.asset_name
        
        return {'FINISHED'}

class LgnPullAssetOperator(bpy.types.Operator):
    bl_idname = "lgn.pull_asset"
    bl_label = "Import asset"

    def execute(self, context):
        server_address = get_preferences(context).server_address
        print("connecting to " + server_address)
        
        with sonora.client.insecure_web_channel(server_address) as channel:
            sc_stub = source_control_pb2_grpc.SourceControlStub(channel)
            selected_asset_name = context.scene.pull_properties.assets[context.scene.pull_properties.assets_index].id
            response = sc_stub.PullDCCAsset(source_control_pb2.PullDCCAssetRequest(id=selected_asset_name))
            filename = "{}/{}.glb".format(bpy.app.tempdir, selected_asset_name)
            with open(filename, "wb") as file:
                file.write(response.content)
            bpy.ops.import_scene.gltf(filepath=filename)
        
        return {'FINISHED'}

class VIEW3D_PT_list_assets(bpy.types.Panel):
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = "Legion"
    bl_label = "List assets"

    def draw(self, context):
        layout = self.layout
        box = layout.box()
        box.prop(context.scene.pull_properties, "assets")
        scene = context.scene
        box.row().template_list(
            "Lgn_UL_ObjectsToImport", "pull_assets",
            scene.pull_properties, "assets",
            scene.pull_properties, "assets_index"
        )
        props = layout.operator("lgn.list_assets_operator")
        props = layout.operator("lgn.pull_asset")


def register():
    bpy.utils.register_class(LgnListAssetsOperator)
    bpy.utils.register_class(LgnPullAssetOperator)
    bpy.utils.register_class(VIEW3D_PT_list_assets)
    bpy.utils.register_class(AssetProperties)
    bpy.utils.register_class(PullProperties)
    bpy.utils.register_class(Lgn_UL_ObjectsToImport)
    bpy.types.Scene.pull_properties = bpy.props.PointerProperty(type=PullProperties)
    
def unregister():
    bpy.utils.unregister_class(LgnListAssetsOperator)
    bpy.utils.unregister_class(LgnPullAssetOperator)
    bpy.utils.unregister_class(VIEW3D_PT_list_assets)
    bpy.utils.unregister_class(PullProperties)
    bpy.utils.unregister_class(Lgn_UL_ObjectsToImport)
    bpy.utils.unregister_class(AssetProperties)