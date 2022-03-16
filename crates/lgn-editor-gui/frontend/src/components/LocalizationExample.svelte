<script lang="ts">
  import Select from "./inputs/Select.svelte";
  import TextInput from "./inputs/TextInput.svelte";
  import NumberInput from "./inputs/NumberInput.svelte";
  import { availableLocales, locale, t } from "@/stores/l10n";

  let userName = "Foo";

  let photoCount = 0;

  let userGender = "other";
</script>

<div class="bg-gray-700 h-full w-full flex flex-col px-2 space-y-2">
  <div>
    {$t("hello-user", { userName })}
  </div>

  <div>
    {$t("shared-photos", { userName, photoCount: photoCount || 0, userGender })}
  </div>

  <Select
    on:select={({ detail: newLocale }) =>
      newLocale && ($locale = newLocale.value)}
    value={{ item: $locale, value: $locale }}
    options={$availableLocales.map((locale) => ({
      item: locale,
      value: locale,
    }))}
  >
    <div slot="option" let:option>{option.item}</div>
  </Select>

  <TextInput bind:value={userName} />

  <NumberInput min={0} noArrow bind:value={photoCount} />

  <Select
    on:select={({ detail: newUserGender }) =>
      newUserGender && (userGender = newUserGender.value)}
    value={{ item: userGender, value: userGender }}
    options={["female", "non-binary", "male", "other"].map((gender) => ({
      item: gender,
      value: gender,
    }))}
  >
    <div slot="option" let:option>{option.item}</div>
  </Select>
</div>
