import os
import bpy
from connection import Connection
import resource_browser_pb2_grpc
import resource_browser_pb2
import source_control_pb2_grpc
import source_control_pb2
from preferences import get_preferences


class Lgn_UL_ObjectsToExport(bpy.types.UIList):
    def draw_item(self, context, layout, data, item, icon, active_data, active_propname):
        layout.prop(item, "name")
    
class PushProperties(bpy.types.PropertyGroup):
    asset_name: bpy.props.StringProperty(
        name="Asset Name",
        default="example"
    )

class LgnPushOperator(bpy.types.Operator):
    bl_idname = "lgn.push_operator"
    bl_label = "Export data to Legion Engine"

    def execute(self, context):
        asset_name = context.scene.push_properties.asset_name
        filename = "{}/{}.glb".format(bpy.app.tempdir, asset_name)
        bpy.ops.export_scene.gltf(filepath = filename, export_selected=True)
        filesize = os.path.getsize(filename)

        with Connection(context) as conn:
            sc_stub = source_control_pb2_grpc.SourceControlStub(conn.channel)
            init_response = sc_stub.InitUploadRawFile(source_control_pb2.InitUploadRawFileRequest(name="{}.glb".format(asset_name), size=filesize), timeout=conn.timeout)
            print("Upload init response: {response}".format(response=init_response))
            if init_response.status == source_control_pb2.QUEUED:
                file = open(filename, "rb")
                for upload_response in sc_stub.UploadRawFile(source_control_pb2.UploadRawFileRequest(id=init_response.id, content=file.read()), timeout=conn.timeout):
                    if getattr(upload_response, upload_response.WhichOneof('response')).status == source_control_pb2.DONE:
                        print("Upload response: {response}".format(response=upload_response))
                        rb_stub = resource_browser_pb2_grpc.ResourceBrowserStub(conn.channel)
                        rb_stub.CreateResource(resource_browser_pb2.CreateResourceRequest(resource_type="gltf", upload_id=init_response.id, resource_name="{}.glb".format(asset_name)), timeout=conn.timeout)
                        context.window_manager.popup_menu(LgnPushOperator.draw_finished)
            else:
                print("Upload init failed: {response}".format(response=init_response))
        
        return {'FINISHED'}

    def draw_finished(self, context):
        self.layout.label(text="Model exported successfully")

    def draw(self, context):
        layout = self.layout
        layout.prop(self, "asset_name")
        
class VIEW3D_PT_export_data(bpy.types.Panel):
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = "Legion"
    bl_label = "Export data"

    def draw(self, context):
        layout = self.layout
        selected_objects = bpy.context.selected_objects
        box = layout.box()
        for o in selected_objects:
            box.label(text="{name} {type}".format(name = o.name, type = o.type))
        layout.prop(bpy.context.scene.push_properties, "asset_name")
        layout.operator("lgn.push_operator")


def register():
    bpy.utils.register_class(LgnPushOperator)
    bpy.utils.register_class(Lgn_UL_ObjectsToExport)
    bpy.utils.register_class(VIEW3D_PT_export_data)
    bpy.utils.register_class(PushProperties)
    bpy.types.Scene.push_properties = bpy.props.PointerProperty(type=PushProperties)

def unregister():
    bpy.utils.unregister_class(LgnPushOperator)
    bpy.utils.unregister_class(Lgn_UL_ObjectsToExport)
    bpy.utils.unregister_class(VIEW3D_PT_export_data)
    bpy.utils.unregister_class(PushProperties)