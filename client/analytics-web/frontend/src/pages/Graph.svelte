<script lang="ts">

  type GraphParams = {
    process_id: string | null;
    start_ms: number;
    end_ms: number;
  };
  
  import { useLocation } from "svelte-navigator";
  const location = useLocation();

  function getUrlParams() :GraphParams | null {
    const params = new URLSearchParams($location.search);
    const process_id = params.get("process")
    if (!process_id) {
      return null;
    }
    const start_str = params.get("start");
    if (!start_str) {
      return null;
    }
    const end_str = params.get("end");
    if (!end_str) {
      return null;
    }
    return {
      process_id : process_id,
      start_ms: parseFloat(start_str),
      end_ms: parseFloat(end_str)
    };
  }

  console.log(getUrlParams());
  
</script>

<div>
  <h1>Graph</h1>
</div>
