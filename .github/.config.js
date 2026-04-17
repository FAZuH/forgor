// Credit: Workflow configs based on https://github.com/Wynntils/Wynntils
//
// https://github.com/conventional-changelog/conventional-changelog-config-spec/blob/master/versions/2.2.0/README.md
"use strict";
const config = require("conventional-changelog-conventionalcommits");

// chore!(major) -> major (0)
// chore!(minor) -> minor (1)
// otherwise -> patch (2)
function whatBump(commits) {
  const hasMajor = commits.some(c => c?.header?.startsWith("chore!(major)"));
  const hasMinor = commits.some(c => c?.header?.startsWith("chore!(minor)"));

  if (hasMajor) {
    return {
      releaseType: "major",
      reason: "Found a commit with a chore!(major) type."
    };
  }

  if (hasMinor) {
    return {
      releaseType: "minor",
      reason: "Found a commit with a chore!(minor) type."
    };
  }

  return {
    releaseType: "patch",
    reason: "No special commits found. Defaulting to a patch."
  };
}

// Check if commit should appear in changelog (has [public] marker)
function isPublicCommit(commit) {
  const publicMarker = /\[public\]/i;
  const header = commit.header || "";
  const body = commit.body || "";
  const subject = commit.subject || "";

  return publicMarker.test(header) || publicMarker.test(body) || publicMarker.test(subject);
}

async function getOptions() {
  let options = await config({
    types: [
      { type: "feat", section: "New Features" },
      { type: "fix", section: "Bug Fixes" },
      { type: "perf", section: "Performance Improvements" },
      { type: "docs", section: "Documentation" },
      { type: "revert", section: "Reverts" },

      { type: "style", section: "Styles" },
      { type: "chore", section: "Miscellaneous Chores" },
      { type: "refactor", section: "Code Refactoring" },
      { type: "test", section: "Tests" },
      { type: "build", section: "Build System" },
      { type: "ci", section: "Continuous Integration" },
    ],
  });

  // Both of these are used in different places...
  options.recommendedBumpOpts.whatBump = whatBump;
  options.whatBump = whatBump;

  if (options.writerOpts && options.writerOpts.transform) {
    const originalTransform = options.writerOpts.transform;
    options.writerOpts.transform = (commit, context) => {
      // Filter out non-public commits (return null to exclude)
      if (!isPublicCommit(commit)) {
        return null;
      }

      // Remove [public] marker and [skip ci] from display
      const publicMarker = /\s*\[public\]/gi;
      const skipCiRegex = /\s*\[skip ci\]/gi;

      if (commit.header) {
        commit.header = commit.header.replace(publicMarker, "").replace(skipCiRegex, "").trim();
      }
      if (commit.subject) {
        commit.subject = commit.subject.replace(publicMarker, "").replace(skipCiRegex, "").trim();
      }

      return originalTransform(commit, context);
    };
  }

  return options;
}

module.exports = getOptions();
