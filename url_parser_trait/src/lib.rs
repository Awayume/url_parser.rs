// SPDX-FileCopyrightText: 2023 Awayume <dev@awayume.jp>
// SPDX-License-Identifier: Apache-2.0

pub trait QueryParams {
    fn to_query_params(&self) -> String;
}
