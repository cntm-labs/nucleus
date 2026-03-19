export function BillingPage() {
  return (
    <div>
      <h1 className="text-2xl font-bold mb-6">Billing</h1>
      <div className="bg-white rounded-xl border border-gray-200 p-6 mb-6">
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="text-lg font-semibold">Pro Plan</h2>
            <p className="text-sm text-gray-500">$25/month</p>
          </div>
          <button className="px-4 py-2 border border-gray-300 rounded-lg text-sm hover:bg-gray-50">Change Plan</button>
        </div>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="text-sm text-gray-500">MAU Usage</p>
            <div className="flex items-end gap-1 mt-1">
              <span className="text-xl font-bold">890</span>
              <span className="text-sm text-gray-400 mb-0.5">/ 10,000</span>
            </div>
            <div className="h-2 bg-gray-100 rounded-full mt-2">
              <div className="h-2 bg-nucleus-500 rounded-full" style={{ width: '8.9%' }} />
            </div>
          </div>
          <div>
            <p className="text-sm text-gray-500">API Requests</p>
            <div className="flex items-end gap-1 mt-1">
              <span className="text-xl font-bold">45.2K</span>
              <span className="text-sm text-gray-400 mb-0.5">/ 1M</span>
            </div>
            <div className="h-2 bg-gray-100 rounded-full mt-2">
              <div className="h-2 bg-nucleus-500 rounded-full" style={{ width: '4.5%' }} />
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
