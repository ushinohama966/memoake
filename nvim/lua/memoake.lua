-- memoake.lua
local M = {}

-- ------------------------------------------------------------------
-- ユーティリティ: CLIを実行してJSONを取得
-- ------------------------------------------------------------------
local function fetch_memos(cb)
	vim.system({ "memoake-cli", "list-json" }, { text = true }, function(obj)
		if obj.code ~= 0 then
			vim.schedule(function()
				vim.notify("Failed to fetch memos: " .. (obj.stderr or ""), vim.log.levels.ERROR)
			end)
			return
		end

		vim.schedule(function()
			local ok, memos = pcall(vim.json.decode, obj.stdout)
			if ok and type(memos) == "table" then
				cb(memos)
			else
				cb({})
			end
		end)
	end)
end

-- ------------------------------------------------------------------
-- Lazygit 風 TUI メイン UI
-- ------------------------------------------------------------------
function M.open_tui()
	fetch_memos(function(memos)
		-- 画面サイズの計算
		local total_width = math.floor(vim.o.columns * 0.85)
		local total_height = math.floor(vim.o.lines * 0.75)
		local row = math.floor((vim.o.lines - total_height) / 2)
		local col = math.floor((vim.o.columns - total_width) / 2)

		local list_width = math.floor(total_width * 0.4)
		local preview_width = total_width - list_width - 2 -- 枠線やマージン分

		-- 1. バッファの作成
		local list_buf = vim.api.nvim_create_buf(false, true)
		local preview_buf = vim.api.nvim_create_buf(false, true)

		-- 2. 左側：リストウィンドウ
		local list_win = vim.api.nvim_open_win(list_buf, true, {
			relative = "editor",
			width = list_width,
			height = total_height,
			row = row,
			col = col,
			style = "minimal",
			border = "rounded",
			title = " Memos (a:add, d:del, e:edit, q:quit) ",
			title_pos = "center",
		})

		-- 3. 右側：プレビューウィンドウ
		local preview_win = vim.api.nvim_open_win(preview_buf, false, {
			relative = "editor",
			width = preview_width,
			height = total_height,
			row = row,
			col = col + list_width + 2,
			style = "minimal",
			border = "rounded",
			title = " Preview ",
			title_pos = "center",
		})

		-- ハイライト等の設定
		vim.wo[list_win].cursorline = true
		vim.bo[list_buf].filetype = "memoake-list"
		vim.bo[preview_buf].filetype = "markdown"

		-- 表示用テキストの生成
		local list_lines = {}
		for _, m in ipairs(memos) do
			local line = m.content:gsub("\n", " ")
			table.insert(list_lines, string.format("[%d] %s", m.id, line))
		end

		if #list_lines == 0 then
			table.insert(list_lines, "（メモがありません）")
		end

		vim.api.nvim_buf_set_lines(list_buf, 0, -1, false, list_lines)

		-- --------------------------------------------------------------
		-- プレビュー更新処理
		-- --------------------------------------------------------------
		local function update_preview()
			if #memos == 0 then
				vim.api.nvim_buf_set_lines(preview_buf, 0, -1, false, { "メモが選択されていません" })
				return
			end

			local cursor = vim.api.nvim_win_get_cursor(list_win)
			local idx = cursor[1]
			local selected = memos[idx]

			if selected then
				local preview_text = {
					"# Memo ID: " .. selected.id,
					"**Created:** " .. selected.created_at,
					"---",
					"",
				}
				for _, line in ipairs(vim.split(selected.content, "\n")) do
					table.insert(preview_text, line)
				end
				vim.api.nvim_buf_set_lines(preview_buf, 0, -1, false, preview_text)
			end
		end

		-- 初期表示のプレビュー更新
		update_preview()

		-- カーソルが移動するたびにプレビューを非同期更新 (lazygit感)
		local group = vim.api.nvim_create_augroup("MemoakePreviewGroup", { clear = true })
		vim.api.nvim_create_autocmd({ "CursorMoved", "CursorMovedI" }, {
			group = group,
			buffer = list_buf,
			callback = update_preview,
		})

		-- ウィンドウを安全に閉じるヘルパー
		local function close_all()
			pcall(vim.api.nvim_del_augroup_by_id, group)
			if vim.api.nvim_win_is_valid(list_win) then
				vim.api.nvim_win_close(list_win, true)
			end
			if vim.api.nvim_win_is_valid(preview_win) then
				vim.api.nvim_win_close(preview_win, true)
			end
		end

		-- --------------------------------------------------------------
		-- キーバインド設定 (lazygit 操作感)
		-- --------------------------------------------------------------
		local opts = { buffer = list_buf, silent = true }

		-- q / <Esc>: 閉じる
		vim.keymap.set("n", "q", close_all, opts)
		vim.keymap.set("n", "<Esc>", close_all, opts)

		-- d: 選択中メモの削除
		vim.keymap.set("n", "d", function()
			if #memos == 0 then
				return
			end
			local cursor = vim.api.nvim_win_get_cursor(list_win)
			local selected = memos[cursor[1]]
			if not selected then
				return
			end

			local confirm = vim.fn.confirm("Memo [" .. selected.id .. "] を削除しますか？", "&Yes\n&No", 2)
			if confirm == 1 then
				vim.system({ "memoake-cli", "delete", tostring(selected.id) }, {}, function(obj)
					vim.schedule(function()
						close_all()
						M.open_tui() -- リロードして再描画
					end)
				end)
			end
		end, opts)

		-- a: 新規メモ追加
		vim.keymap.set("n", "a", function()
			close_all()
			M.open_memoake()
		end, opts)
	end)
end

-- ------------------------------------------------------------------
-- 従来通り: フローティング入力ウィンドウ (新規作成用)
-- ------------------------------------------------------------------
function M.open_memoake()
	local width = 60
	local height = 6
	local buf = vim.api.nvim_create_buf(false, true)

	local win = vim.api.nvim_open_win(buf, true, {
		relative = "editor",
		width = width,
		height = height,
		col = (vim.o.columns - width) / 2,
		row = (vim.o.lines - height) / 2,
		style = "minimal",
		border = "rounded",
		title = " New Memo (<Ctrl+S> to Save) ",
		title_pos = "center",
	})

	-- Ctrl+S または Enter で保存
	local function save()
		local lines = vim.api.nvim_buf_get_lines(buf, 0, -1, false)
		local text = table.concat(lines, "\n")

		if #text > 0 then
			vim.system({ "memoake-cli", "create", text }, { text = true }, function(obj)
				vim.schedule(function()
					if obj.code == 0 then
						vim.notify("Memo saved!", vim.log.levels.INFO)
						M.open_tui() -- 保存後に Lazygit UI を開く
					end
				end)
			end)
		end
		if vim.api.nvim_win_is_valid(win) then
			vim.api.nvim_win_close(win, true)
		end
	end

	vim.keymap.set({ "n", "i" }, "<C-s>", save, { buffer = buf, silent = true })
	vim.cmd("startinsert")
end

-- キーバインド登録
vim.keymap.set("n", "<leader>ma", M.open_memoake, { desc = "Memoake: Add" })
vim.keymap.set("n", "<leader>ml", M.open_tui, { desc = "Memoake: Lazygit TUI" })

return M
