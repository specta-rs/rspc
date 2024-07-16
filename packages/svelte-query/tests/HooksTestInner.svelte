<script lang="ts">
import { rspc } from "./utils";

const query = rspc.nested.procedures.one.createQuery("test");

const mutation = rspc.nested.procedures.two.createMutation();
$mutation.mutate("test");

rspc.nested.procedures.three.createSubscription("value", {
	onData: () => {},
});
</script>

<!-- necessary for fetch to actually happen -->
{JSON.stringify($query.data)}
