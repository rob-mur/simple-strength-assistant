import semanticRelease from 'semantic-release';
import { appendFileSync } from 'fs';

try {
  const result = await semanticRelease();

  if (result) {
    const { nextRelease, commits } = result;
    console.log(`Published ${nextRelease.type} release version ${nextRelease.version} containing ${commits.length} commits.`);

    if (process.env.GITHUB_OUTPUT) {
      appendFileSync(process.env.GITHUB_OUTPUT, `new_release_published=true\n`);
      appendFileSync(process.env.GITHUB_OUTPUT, `new_release_version=${nextRelease.version}\n`);
      appendFileSync(process.env.GITHUB_OUTPUT, `new_release_git_tag=${nextRelease.gitTag}\n`);
    }
  } else {
    console.log('No release published.');
    if (process.env.GITHUB_OUTPUT) {
        appendFileSync(process.env.GITHUB_OUTPUT, `new_release_published=false\n`);
    }
  }
} catch (err) {
  console.error('The automated release failed with %O', err);
  process.exit(1);
}
