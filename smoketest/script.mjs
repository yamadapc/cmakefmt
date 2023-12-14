// zx

async function writePage(items) {
  for (let item of items) {
    const pth = item.path;
    const repository = item.repository.full_name;
    const ref = item.url.split('?')[1].split('ref=')[1];
    const rawUrl = `https://raw.githubusercontent.com/${repository}/${ref}/${pth}`;
    const targetPath = `tests/${repository}/${path.dirname(pth)}`;
    if (await fs.exists(`tests/${repository}/${pth}`)) {
      continue;
    }
    await $`mkdir -p ${targetPath}`;
    const text = await (await fetch(rawUrl)).text();
    await fs.writeFile(`tests/${repository}/${pth}`, text);
  }
}

async function fetchPage(page = 0) {
  const output = JSON.parse(await $`gh api /search/code -X GET -f 'q=language:cmake project' -f per_page=100 -f page=${page}`.quiet());
  const totalCount = output.total_count;
  const items = output.items;
  console.log((page * 100) + items.length, '/', totalCount);
  return { items, totalCount };
}

const firstPage = await fetchPage();
await writePage(firstPage.items);
let totalCount = firstPage.totalCount - 100;
let currentPage = 2;
while (totalCount > 0) {
  const page = await fetchPage(currentPage);
  await writePage(page.items);
  currentPage += 1;
  totalCount -= 100;
}
