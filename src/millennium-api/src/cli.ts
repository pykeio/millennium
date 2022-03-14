/**
 * Copyright 2022 pyke.io
 *           2019-2021 Tauri Programme within The Commons Conservancy
 *                     [https://tauri.studio/]
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/**
 * Parse arguments from the application's command line.
 *
 * @module
 */

import { invokeMillenniumCommand } from './helpers/millennium';

export interface ArgMatch {
	value: string | boolean | string[] | null;
	occurrences: number;
}

export interface SubcommandMatch {
	name: string;
	matches: CliMatches;
}

export interface CliMatches {
	args: { [name: string]: ArgMatch };
	subcommand: SubcommandMatch | null;
}

/**
 * Parse the arguments provided to the application process and get the matches using the
 * configuration defined under `.millenniumrc > millennium > cli`.
 */
export async function getMatches(): Promise<CliMatches> {
	return await invokeMillenniumCommand<CliMatches>({
		__millenniumModule: 'Cli',
		message: {
			cmd: 'cliMatches'
		}
	});
}
