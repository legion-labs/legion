
struct CopyJob
{
    uint source_offset;
    uint dest_offset;
    uint size;
}

StructuredBuffer<CopyJob> copy_job_buffer;

ByteAddressBuffer transient_buffer;
RWByteAddressBuffer static_buffer;

[numthreads(1, 1, 1)]
void CSMain(uint3 id : SV_DispatchThreadID)
{
    uint src = copy_job_buffer[id.x].source_offset;
    uint dst = copy_job_buffer[id.x].dest_offset;

    for (uint i = 0; i < copy_job_buffer[id.x].size; i++)
    {
        static_buffer.Store(dst, transient_buffer.Load(src));
    }
}
